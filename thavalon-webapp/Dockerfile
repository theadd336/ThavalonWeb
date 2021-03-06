# syntax=docker/dockerfile:experimental
FROM node:latest as builder

WORKDIR /app
COPY ./thavalon-webapp /app

RUN --mount=type=cache,target=/app/node_modules \
  npm install && npm run build && mv /app/build /srv/build-output

FROM nginx:latest

COPY --from=builder /srv/build-output /usr/share/nginx/html
COPY k8s/webapp/default.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]