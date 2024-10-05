FROM ubuntu:24.04 AS cert-generator

RUN apt-get update && \
    apt-get install -y openssl && \
    mkdir -p /certs && \
    openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
    -keyout /certs/tls.key \
    -out /certs/tls.crt \
    -subj "/CN=localhost"

FROM nginx:alpine AS web

WORKDIR /usr/share/nginx/html

COPY ./dist ./

COPY --from=cert-generator /certs/tls.key /etc/ssl/certs/tls.key
COPY --from=cert-generator /certs/tls.crt /etc/ssl/certs/tls.crt

ADD ./config/nginx/default.conf /etc/nginx/conf.d/default.conf

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]