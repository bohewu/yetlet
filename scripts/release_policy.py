"""Committed workflow policy: automated workers, one human publication gate."""
import os


def require(value):
    if not value:
        raise ValueError('release policy rejected')


def automatic():
    mode = os.environ.get('RELEASE_AUTOMATION')
    require(mode in (None, 'single-publication-v1'))
    return mode == 'single-publication-v1'


def reviewers(environment, publication=False):
    rules = [r for r in environment['protection_rules'] if r['type'] == 'required_reviewers']
    if automatic() and not publication:
        require(not rules)
        return
    require(len(rules) == 1 and rules[0].get('prevent_self_review', False) is False)
    require(len(rules[0]['reviewers']) == 1)
    reviewer = rules[0]['reviewers'][0]
    require(reviewer['type'] == 'User' and reviewer['reviewer']['login'] == 'bohewu'
            and reviewer['reviewer']['id'] == 35250002)


def ci(source, api):
    """Green protected-master CI is required independently of signature success."""
    runs = api('/actions/workflows/rust.yml/runs?head_sha=' + source + '&event=push&per_page=100')
    candidates = [r for r in runs['workflow_runs'] if r['head_sha'] == source
        and r['head_branch'] == 'master' and r['event'] == 'push' and r['status'] == 'completed'
        and r['conclusion'] == 'success' and r['repository']['id'] == 1409587171]
    require(candidates)
    run = max(candidates, key=lambda r: r['id'])
    jobs = api('/actions/runs/' + str(run['id']) + '/attempts/' + str(run['run_attempt']) + '/jobs?per_page=100')
    expected = {'Rust 1.99 MSRV', 'Stable native, samples, and Chrome'}
    require(jobs['total_count'] == len(jobs['jobs']) and
        {j['name'] for j in jobs['jobs']} == expected)
    require(all(j['status'] == 'completed' and j['conclusion'] == 'success' for j in jobs['jobs']))
    return {'run': str(run['id']), 'attempt': str(run['run_attempt']), 'source': source}
