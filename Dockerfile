FROM nginx:alpine AS web

WORKDIR /usr/share/nginx/html

COPY ./packages/gh-pagify/dist .

ADD ./config/nginx/default.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]