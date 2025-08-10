# {username} | {uptime}

echo "$(whoami) | Up $(uptime -p | sed 's/up //')"