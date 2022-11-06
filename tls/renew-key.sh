## renew key and convert to pkcs

sudo certbot renew

sudo openssl pkcs12 -export -in /etc/letsencrypt/live/ws.gh.maygoo.au/fullchain.pem -inkey /etc/letsencrypt/live/ws.gh.maygoo.au/privkey.pem -out ~/board-games-rust/tls/keystore.pkcs