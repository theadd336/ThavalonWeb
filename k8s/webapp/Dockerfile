FROM node:latest as build
WORKDIR /app
COPY ./thavalon-webapp/package.json .
RUN npm install
RUN npm install react-scripts -g
COPY ./thavalon-webapp .
RUN npm run build
COPY ./k8s/webapp/default.conf .

FROM nginx:latest
COPY --from=build /app/build /usr/share/nginx/html
COPY --from=build /app/default.conf /etc/nginx/conf.d/default.conf
CMD ["nginx", "-g", "daemon off;"]