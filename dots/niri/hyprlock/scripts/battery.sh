batteryPercentage=$(
  upower -i /org/freedesktop/UPower/devices/battery_BAT1 |
  grep "percentage:" |
  awk '{print $2}' |
  sed 's/%//'
)

if [ $batteryPercentage -lt 20 ]; then
  echo "<span foreground='#ff2222'>$batteryPercentage%</span>"
elif [ $batteryPercentage -lt 40 ]; then
  echo "<span foreground='#ffaa22'>$batteryPercentage%</span>"
else
  echo "<span foreground='#22ff22'>$batteryPercentage%</span>"
fi