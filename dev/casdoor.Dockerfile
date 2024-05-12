FROM casbin/casdoor:v1.595.0
USER root
COPY dev-public.pem /usr/local/share/ca-certificates/dev-public.crt
RUN update-ca-certificates