# Qt applications follow the GTK theme (dark or light): Qt chooses its GTK
# platform theme by itself on XFCE, GNOME or MATE, not on Openbox (7.0-54)
[ -z "$QT_QPA_PLATFORMTHEME" ] && export QT_QPA_PLATFORMTHEME=gtk3
