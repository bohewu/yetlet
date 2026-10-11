-- Private single-primary experiment. All keys are supplied explicitly.
-- KEYS: record, independent operation result, replacement, result admission index.
-- ARGV: kind, digest, expected revision, opaque state, CSRF, dispatch deadline,
--       session TTL, operation TTL, tombstone TTL, payload limit, result cap.
local kind, digest = ARGV[1], ARGV[2]
local clock = redis.call('TIME')
local now = tonumber(clock[1]) * 1000 + math.floor(tonumber(clock[2]) / 1000)
local function keytype(key) return redis.call('TYPE', key).ok end
local function result()
    if keytype(KEYS[2]) ~= 'hash' then return {'unresolved', '0'} end
    local r = redis.call('HMGET', KEYS[2], 'kind', 'digest', 'status', 'revision')
    local expected_kind = kind == 'resolve' and ARGV[12] or kind
    if r[1] ~= expected_kind then return {'unresolved', '0'} end
    if r[2] ~= digest or not r[3] or not r[4] then return {'unresolved', '0'} end
    return {r[3], r[4]}
end
if kind == 'resolve' then return result() end

local limit = tonumber(ARGV[10])
local function record()
    local t = keytype(KEYS[1])
    if t == 'none' then return {'missing', '0'} end
    if t ~= 'hash' then return {'invalid', '0'} end
    for _, field in ipairs({'schema', 'revision', 'status', 'expires', 'generation'}) do
        if redis.call('HSTRLEN', KEYS[1], field) > 16 then return {'invalid', '0'} end
    end
    local m = redis.call('HMGET', KEYS[1], 'schema', 'revision', 'status', 'expires', 'generation')
    if m[1] ~= '1' or not tonumber(m[2]) or not tonumber(m[4]) or not tonumber(m[5]) then
        return {'invalid', '0'}
    end
    if m[3] == 'rotated' or m[3] == 'revoked' then return {m[3], m[2]} end
    if m[3] ~= 'active' then return {'invalid', '0'} end
    if tonumber(m[4]) <= now then return {'expired', m[2]} end
    if redis.call('HEXISTS', KEYS[1], 'state') == 0 or redis.call('HSTRLEN', KEYS[1], 'csrf') ~= 64
        or redis.call('HSTRLEN', KEYS[1], 'state') > limit then return {'invalid', m[2]} end
    return {'ok', m[2], redis.call('HGET', KEYS[1], 'state'),
        redis.call('HGET', KEYS[1], 'csrf'), m[4], m[5]}
end
if kind == 'read' then return record() end
if kind ~= 'create' and kind ~= 'update' and kind ~= 'rotate'
    and kind ~= 'replace' and kind ~= 'revoke' then return {'invalid', '0'} end

-- Validate before any state writes. Runtime/OOM errors still mean unknown;
-- Redis scripts provide isolation, not rollback on a runtime error.
local deadline, ttl = tonumber(ARGV[6]), tonumber(ARGV[7])
local operation_ttl, tombstone_ttl, cap = tonumber(ARGV[8]), tonumber(ARGV[9]), tonumber(ARGV[11])
local expected = tonumber(ARGV[3])
if not deadline or not ttl or not operation_ttl or not tombstone_ttl or not cap
    or not limit or not expected or ttl < 1 or ttl > 60000 or operation_ttl < 2000
    or operation_ttl > 60000 or tombstone_ttl < 1 or tombstone_ttl > 60000
    or limit < 1 or limit > 4096 or cap < 1 or cap > 128
    or expected < 0 or expected > 1000000 or #ARGV[4] > limit then return {'invalid', '0'} end
if now > deadline or deadline > now + 1000 then return {'deadline', '0'} end
if keytype(KEYS[2]) ~= 'none' then return result() end
local index_type = keytype(KEYS[4])
if index_type ~= 'none' and index_type ~= 'zset' then return {'invalid', '0'} end
redis.call('ZREMRANGEBYSCORE', KEYS[4], '-inf', now)
if redis.call('ZCARD', KEYS[4]) >= cap then return {'capacity', '0'} end
local function finish(status, revision)
    redis.call('HSET', KEYS[2], 'kind', kind, 'digest', digest,
        'status', status, 'revision', tostring(revision))
    redis.call('PEXPIREAT', KEYS[2], now + operation_ttl)
    redis.call('ZADD', KEYS[4], now + operation_ttl, KEYS[2])
    redis.call('PEXPIRE', KEYS[4], operation_ttl)
    return {status, tostring(revision)}
end
local function activate(key, revision, generation, state, csrf, expires)
    redis.call('HSET', key, 'schema', '1', 'revision', revision, 'status', 'active',
        'state', state, 'csrf', csrf, 'expires', expires, 'generation', generation)
    redis.call('PEXPIREAT', key, expires)
end
if kind == 'create' then
    if keytype(KEYS[1]) ~= 'none' then return finish('conflict', 0) end
    activate(KEYS[1], 0, 0, ARGV[4], ARGV[5], now + ttl)
    return finish('ok', 0)
end
local r = record()
if r[1] ~= 'ok' then return finish(r[1], tonumber(r[2])) end
local revision = tonumber(r[2])
if revision ~= expected then return finish('conflict', revision) end
if revision >= 1000000 then return finish('invalid', revision) end
if kind == 'update' then
    redis.call('HSET', KEYS[1], 'state', ARGV[4], 'revision', revision + 1)
    return finish('ok', revision + 1)
end
if kind == 'rotate' or kind == 'replace' then
    if KEYS[1] == KEYS[3] or keytype(KEYS[3]) ~= 'none' then return finish('conflict', revision) end
    local state = r[3]
    if kind == 'replace' then state = ARGV[4] end
    activate(KEYS[3], 0, tonumber(r[6]) + 1, state, ARGV[5], now + ttl)
end
-- Tombstones contain neither state/CSRF nor the replacement identity.
redis.call('DEL', KEYS[1])
redis.call('HSET', KEYS[1], 'schema', '1', 'revision', revision + 1,
    'generation', r[6], 'expires', now + tombstone_ttl,
    'status', kind == 'revoke' and 'revoked' or 'rotated')
redis.call('PEXPIRE', KEYS[1], tombstone_ttl)
return finish('ok', revision + 1)
