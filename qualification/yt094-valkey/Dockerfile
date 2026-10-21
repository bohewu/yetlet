# The local preparation step supplies only the compiled app, its runtime
# libraries, public settings and the existing frontend build. No source/secrets.
FROM scratch
COPY rootfs/ /
USER 10001:10001
ENTRYPOINT ["/app/admin-dashboard"]
