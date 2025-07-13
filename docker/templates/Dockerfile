FROM nginx:latest
COPY nginx.conf     /etc/nginx/conf.d/default.conf
COPY fullchain.pem  /etc/certs/fullchain.pem
COPY privkey.pem    /etc/certs/privkey.pem
COPY www            /var/www/

CMD ["nginx", "-g", "daemon off;"]