#!/usr/bin/env bash

set -eu
set -o pipefail

# https://www.packer.io/docs/other/debugging.html#issues-installing-ubuntu-packages
while [ ! -f /var/lib/cloud/instance/boot-finished ]; do echo 'Waiting for cloud-init...'; sleep 1; done

cd /home/ubuntu 

apt update -o Acquire::Check-Valid-Until=false

apt install --yes \
apache2 \
mysql-server \
php7.2 \
php7.2-mysql \
php7.2-xml \
php7.2-gd

wget https://ftp.drupal.org/files/projects/drupal-8.5.0.tar.gz
tar xvf drupal-8.5.0.tar.gz

echo "Creating symlink"
ln -s /home/ubuntu/drupal-8.5.0 /var/www/vuln-drupal

echo "Creating Virtual Host"
echo "
<VirtualHost *:80>
    DocumentRoot /var/www/vuln-drupal
    <Directory /var/www>
		Allow from all
		Options FollowSymlinks
		AllowOverride all
	</Directory>
    ErrorLog \${APACHE_LOG_DIR}/error.log
	  CustomLog \${APACHE_LOG_DIR}/access.log combined

    ProxyRequests On
    SSLProxyEngine on

    ProxyPass /news https://www.bbc.co.uk/news
    ProxyPassReverse /news https://www.bbc.co.uk/news
</VirtualHost>
" > /etc/apache2/sites-available/000-default.conf


echo "Setting apache user"
echo "
export APACHE_RUN_USER=ubuntu
export APACHE_RUN_GROUP=ubuntu
" >> /etc/apache2/envvars

echo "creating mysql database and user"
mysql <<EOF
CREATE DATABASE vuln_app;
CREATE USER vuln_app@localhost IDENTIFIED BY 'vuln_app';
GRANT ALL PRIVILEGES ON vuln_app.* TO vuln_app@localhost;
FLUSH PRIVILEGES;
EOF

echo "setting drupal settings"
cp drupal-8.5.0/sites/default/{default.settings,settings}.php
mkdir -p drupal-8.5.0/sites/default/files/config_123/sync

echo "
\$databases['default']['default'] = array (
  'database' => 'vuln_app',
  'username' => 'vuln_app',
  'password' => 'vuln_app',
  'prefix' => '',
  'host' => 'localhost',
  'port' => '3306',
  'namespace' => 'Drupal\\Core\\Database\\Driver\\mysql',
  'driver' => 'mysql',
);
\$settings['install_profile'] = 'standard';
\$config_directories['sync'] = 'sites/default/files/config_123/sync';
\$settings['hash_salt'] = '123';
" >> drupal-8.5.0/sites/default/settings.php

echo "setting ownership of drupal"
chown -R ubuntu:ubuntu /home/ubuntu/drupal-8.5.0

echo "enabling proxy"
a2enmod proxy proxy_http ssl

echo "restarting apache"
systemctl restart apache2

echo "running sql migration"
cat /tmp/vuln_app.sql.gz | gunzip | mysql vuln_app