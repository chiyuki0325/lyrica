import QtQuick 2.15
import QtQuick.Layouts 1.1
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.plasma5support as Plasma5Support
import QtWebSockets

PlasmoidItem {
    id: root
    preferredRepresentation: fullRepresentation
    fullRepresentation: Item {
        id: oneLineLayout
        anchors.fill: parent
        Layout.minimumWidth: text.contentWidth
        Layout.minimumHeight: plasmoid.configuration.layoutHeight
        Layout.preferredWidth: Layout.minimumWidth


        function updateLayoutSize() {
            Layout.minimumWidth = text.contentWidth
        }

        Timer {
            id: timer
        }

        function delay(delayTime,cb) {
            timer.interval = delayTime
            timer.repeat = false
            timer.triggered.connect(cb)
            timer.start()
        }

        WebSocket {
            id: socket
            url: "ws://127.0.0.1:15649/ws"
            onTextMessageReceived: (message) => {
                message = JSON.parse(message)
                switch (message.id) {
                    case 0:
                        // Update music metadata
                        text.text = ""
                        updateLayoutSize()
                        break
                    case 1:
                        // Update lyric line
                        let lyric = message.data.lyric_line.lyric || ""
                        if (lyric.length > plasmoid.configuration.characterLimit) {
                            lyric = lyric.slice(0, plasmoid.configuration.characterLimit) + "..."
                        }
                        text.text = lyric
                        updateLayoutSize()
                        break
                }
            }
            onStatusChanged: (status) => {
                if (status == WebSocket.Closed || status == WebSocket.Error) {
                    if (plasmoid.configuration.showReconnectingText) {
                        text.text = "[" + i18n("Reconnecting") + "...]"
                    } else {
                        text.text = ""
                    }
                    updateLayoutSize()
                    socket.active = false
                    delay(500, () => {
                        if (socket.active == false) {
                            socket.active = true
                        }
                    })
                } else if (status == WebSocket.Open) {
                    // Send config
                    const providerMap = Object.assign({
                        "mpris2_text": 0,
                        "file": 1,
                        "yesplaymusic": 2,
                        "netease_trackid": 3,
                        "feeluown_netease": 4,
                        "netease": 5
                    })
                    const configString = JSON.stringify({
                        verbose: plasmoid.configuration.verbose,
                        tlyric_mode: plasmoid.configuration.tlyricMode,
                        disabled_players: plasmoid.configuration.disabledPlayers.split(","),
                        enabled_lyric_providers: plasmoid.configuration.enabledLyricProviders.split(",").map(p => providerMap[p]),
                        online_search_pattern: plasmoid.configuration.onlineSearchPattern,
                        disabled_folders: plasmoid.configuration.disabledFolders.split("\n"),
                        online_search_timeout: plasmoid.configuration.onlineSearchTimeout,
                        online_search_retry: plasmoid.configuration.onlineSearchRetry,
                    })
                    const xhr = new XMLHttpRequest()
                    xhr.open("POST", "http://127.0.0.1:15649/config/update", true)
                    xhr.setRequestHeader("Content-Type", "application/json")
                    xhr.onreadystatechange = () => {
                        if (xhr.readyState == 4) {
                            console.log(xhr.responseText)
                        }
                    }
                    xhr.send(configString)
                }
            }
            active: false
        }

        Item {
            id: offsetItem
            width: 0
            height: parent.height
            x: 0
            y: 0
        }

        Text {
            property int fontSize: {
             return (plasmoid.configuration.shouldUseDefaultThemeFontSize)
                 ? PlasmaCore.Theme.defaultFont.pixelSize
                 : plasmoid.configuration.configuredFontSize
            }
            id: text
            text: ""
            height: plasmoid.configuration.layoutHeight
            verticalAlignment: Text.AlignVCenter
            font.pixelSize: fontSize
            color: PlasmaCore.Theme.textColor
        }

        Plasmoid.contextualActions: [
            PlasmaCore.Action {
                text: i18nc("Reload configuration", "Reload")
                icon.name: "view-refresh-symbolic"
                priority: PlasmaCore.Action.LowPriority
                onTriggered: {
                    socket.active = false
                    delay(200, () => {
                        socket.active = true
                    })
                }
            }
        ]

	    Plasma5Support.DataSource {
	        id: backendExecutable
		    engine: "executable"
		    connectedSources: []
		    onSourceConnected: {
		        socket.active = true
		    }
	    }

		Component.onCompleted: {
            backendExecutable.connectSource("bash -c '$HOME/.local/share/plasma/plasmoids/ink.chyk.LyricaPlasmoid/contents/bin/lyrica'")
            // TODO: use relative path
		}
    }
}
