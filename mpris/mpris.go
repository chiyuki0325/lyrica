package mpris

import (
	"PlasmaDesktopLyrics/config"
	"github.com/godbus/dbus/v5"
	"strings"
)

func ListPlayers(d *dbus.Conn) []string {
	var result []string
	var players []string
	_ = d.BusObject().Call("org.freedesktop.DBus.ListNames", 0).Store(&result)
	for _, service := range result {
		if strings.HasPrefix(service, "org.mpris.MediaPlayer2.") {
			// 检查是否在黑名单中
			for _, blockedPlayer := range config.Config.DisabledPlayers {
				if strings.HasPrefix(service, "org.mpris.MediaPlayer2."+blockedPlayer) {
					continue
				}
			}
			players = append(players, service)
		}
	}
	return players
}

func GetMetadata(d *dbus.Conn, player string) (map[string]dbus.Variant, bool) {
	dbusObj := d.Object(player, "/org/mpris/MediaPlayer2")
	dbusPropObj, err := dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.Metadata")
	if err != nil {
		return nil, false
	}
	_newMusicUrlVariant := dbusPropObj.Value()
	if _newMusicUrlVariant == nil {
		return nil, false
	}
	return _newMusicUrlVariant.(map[string]dbus.Variant), true
}

func GetPlaybackStatus(d *dbus.Conn, player string) (string, error) {
	dbusObj := d.Object(player, "/org/mpris/MediaPlayer2")
	_dbusResult, err := dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.PlaybackStatus")
	return _dbusResult.Value().(string), err
}

func GetPosition(d *dbus.Conn, player string) int64 {
	dbusObj := d.Object(player, "/org/mpris/MediaPlayer2")
	_dbusResult, err := dbusObj.GetProperty("org.mpris.MediaPlayer2.Player.Position")
	if err != nil {
		return 0
	}
	return _dbusResult.Value().(int64)
}
