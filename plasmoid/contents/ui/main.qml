import QtQuick 2.15
import QtQuick.Layouts 1.1
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
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
            timer.interval = delayTime;
            timer.repeat = false;
            timer.triggered.connect(cb);
            timer.start();
        }

        WebSocket {
            id: socket
            url: "ws://127.0.0.1:15649/ws"
            onTextMessageReceived: function(message) {
                var lyric = JSON.parse(message).lyric || ""
                if (lyric.length > plasmoid.configuration.characterLimit) {
                    lyric = lyric.slice(0, plasmoid.configuration.characterLimit) + "..."
                }
                text.text = lyric
                updateLayoutSize()
                return
            }
            onStatusChanged: function(status) {
                if (status == WebSocket.Closed || status == WebSocket.Error) {
                    text.text = "[" + i18n("Reconnecting") + "...]"
                    updateLayoutSize()
                    socket.active = false
                    delay(500, function() {
                        if (socket.active == false) {
                            socket.active = true
                        }
                    })
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
