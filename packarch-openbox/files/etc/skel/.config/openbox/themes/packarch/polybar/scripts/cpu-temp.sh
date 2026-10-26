#!/usr/bin/env bash

# CPU temperature for polybar, whatever the sensor: k10temp/zenpower (AMD),
# coretemp (Intel), cpu_thermal (ARM), else a thermal zone. Prints nothing
# without a sensor (virtual machine), which hides the module.

# Colors
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
CDIR=$(cd "$DIR" && cd .. && pwd)
WARN=$(grep 'RED' < "$CDIR"/colors.ini | head -n1 | cut -d '=' -f2 | tr -d ' ')
WARN_TEMP=80

# temp*_input of the CPU sensor: Tctl or Package id 0 when labelled
sensor() {
    local h name l
    for h in /sys/class/hwmon/hwmon*; do
        name=$(cat "$h/name" 2> /dev/null)
        case "$name" in
            k10temp|zenpower|coretemp|cpu_thermal) ;;
            *) continue ;;
        esac
        for l in "$h"/temp*_label; do
            case "$(cat "$l" 2> /dev/null)" in
                Tctl|Tdie|"Package id 0") echo "${l%_label}_input"; return ;;
            esac
        done
        [[ -f $h/temp1_input ]] && { echo "$h/temp1_input"; return; }
    done
    for h in /sys/class/thermal/thermal_zone*; do
        case "$(cat "$h/type" 2> /dev/null)" in
            x86_pkg_temp|cpu-thermal|acpitz) echo "$h/temp"; return ;;
        esac
    done
}

input=$(sensor)
[[ -n $input ]] || exit 0
temp=$(( $(cat "$input") / 1000 ))
if (( temp >= WARN_TEMP )); then
    echo "%{F${WARN}}${temp}°C%{F-}"
else
    echo "${temp}°C"
fi
