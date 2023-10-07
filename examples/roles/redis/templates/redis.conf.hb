# this is a handlebars template
# see https://www.jetporch.com/playbooks/using-variables

bind 127.0.0.1 ::1
port {{ redis_port }}
protected-mode yes

timeout 0
tcp-keepalive 300
supervised no

pidfile /var/run/redis/redis-server.pid
loglevel notice
logfile /var/log/redis/redis-server.log

