import QtQuick 2.0
import QtQuick.Layouts 1.1
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core 2.0 as PlasmaCore
import QtWebSockets 1.0

Item {
    id: root
    Plasmoid.preferredRepresentation: Plasmoid.fullRepresentation
    Plasmoid.fullRepresentation: Item {
        id: oneLineLayout
        Layout.minimumWidth: text.contentWidth
        Layout.minimumHeight = text.contentHeight


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

        Text {
            property int fontSize: {
             return (plasmoid.configuration.shouldUseDefaultThemeFontSize)
                 ? PlasmaCore.Theme.defaultFont.pixelSize
                 : plasmoid.configuration.configuredFontSize
            }
            id: text
            text: ""
            verticalAlignment: Text.AlignVCenter
            font.pixelSize: fontSize
            color: PlasmaCore.Theme.textColor
        }

    }
}
