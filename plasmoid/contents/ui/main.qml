import QtQuick 2.15
import QtQuick.Layouts 1.1
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import QtWebSockets

PlasmoidItem {
    id: root
    preferredRepresentation: fullRepres entation
    fullRepresentation: Item {
        id: oneLineLayout
        anchors.fill: parent
        Layout.minimumWidth: text.contentWidth
        Layout.minimumHeight: plasmoid.configuration.layoutHeight
        Layout.preferredWidth: Layout.minimumWidth


        function updateLayoutSize() {
            Layout.minimumWidth = text.contentWidt  h
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
            onTextMessageReceived: function(message) {
                message = JSON.parse(message)
                switch (message.id) {
                    case 0:
                        // Update music metadata
                        text.text = ""
                        updateLayoutSize()
                        break
                    case 1:
                        // Update lyric line
                        var lyric = message.data.lyric_line.lyric || ""
                        if (lyric.length > plasmoid.configuration.characterLimit) {
                            lyric = lyric.slice(0, plasmoid.configuration.characterLimit) + "..."
                        }
                        text.text = lyric
                        updateLayoutSize()
                        break
                }
            }
            onStatusChanged: function(status) {
                if (status == WebSocket.Closed || status == WebSocket.Error) {
                    if (plasmoid.configuration.showReconnectingText) {
                        text.text = "[" + i18n("Reconnecting") + "...]"
                    } else {
                        text.text = ""
                    }
                    updateLayoutSize()
                    socket.active = false
                    delay(500, function() {
                        if (socket.active == false) {
                            socket.active = true
                        }
                    })
                } else if (status == WebSocket.Open) {
                    // Send config
                    var xhr = new XMLHttpRequest()
                    xhr.open("POST", "http://127.0.0.1:15649/config/update", true)
                    xhr.setRequestHeader("Content-Type", "application/json")
                    xhr.send(JSON.stringify({
                        verbose: plasmoid.configuration.verbose,
                        tlyric_mode: plasmoid.configuration.tlyricMode,
                        disabled_players: plasmoid.configuration.disabledPlayers.split(","),
                        enabled_lyric_providers: plasmoid.configuration.enabledLyricProviders.split(","),
                        disabled_folders: plasmoid.configuration.disabledFolders.split("\n")
                    }))
                }
            }
            active: true
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

    }
}
