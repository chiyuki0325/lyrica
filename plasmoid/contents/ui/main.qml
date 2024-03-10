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

        WebSocket {
            id: socket
            url: "ws://127.0.0.1:15648/ws"
            onTextMessageReceived: {
                var lyric = JSON.parse(message).lyric || ""
                if (lyric.length > plasmoid.configuration.characterLimit) {
                    lyric = lyric.slice(0, plasmoid.configuration.characterLimit) + "..."
                }
                text.text = lyric
                updateLayoutSize()
                return
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
