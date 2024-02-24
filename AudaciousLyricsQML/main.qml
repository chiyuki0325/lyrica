import QtQuick 2.15
import QtQuick.Controls 2.15
import QtWebSockets 1.0

ApplicationWindow {
    id: mainWindow
    width: 100
    height: fontSize * 2
    visible: true
    title: qsTr("Audacious Desktop Lyrics")
    color: "transparent"
    flags: Qt.WA_TranslucentBackground | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.SubWindow | Qt.Tool

    property int characterLimit: 40
    property int fontSize: 32
    property string fontColor: "black"
    property string fontOutlineColor: "white"
    property int fontOutlineWidth: 2
    property string fontFamily: "sans-serif"
    property bool fontBold: false

    property string musicName: ""
    property string musicArtist: ""
    property bool showToolTip: false

    function updateLayoutSize() {
        mainWindow.minimumWidth = text.contentWidth + fontSize
    }

    WebSocket {
        id: socket
        url: "ws://127.0.0.1:15648/ws"
        onTextMessageReceived: function(message) {
            var j = JSON.parse(message)
            if (j.title) {
                // Info
                musicName = j.title
                musicArtist = j.artist
                showToolTip = true
            } else {
                if (j.time === -1) {
                    // No-lyric music
                    showToolTip = false
                    text.text = ""
                } else {
                    // Lyric line
                    var lyric = j.lyric || ""
                    if (lyric.length > characterLimit) {
                        lyric = lyric.slice(0, characterLimit) + "..."
                    }
                    text.text = lyric
                    updateLayoutSize()
                }
            }
        }
        active: true
    }

    Text {
        id: text
        text: ""
        verticalAlignment: Text.AlignVCenter
        font.pixelSize: fontSize
        font.bold: fontBold
        font.family: fontFamily
        color: fontColor
        anchors.centerIn: parent
        ToolTip.delay: 1000
        ToolTip.visible: musicName ? showToolTip : false
        ToolTip.text: musicName + "\n" + musicArtist
        Repeater {
            model: 9
            Text {
                x: (index % 3 - 1) * fontOutlineWidth
                y: (Math.floor(index / 3) - 1) * fontOutlineWidth
                text: parent.text
                font.pointSize: parent.font.pointSize
                font.family: parent.font.family
                color: fontOutlineColor
                z: -2
            }
        }
    }

    MouseArea {
        id: toolTipHover
        anchors.fill: parent
        hoverEnabled: true

        onEntered: {
            showToolTip = true
        }

        onExited: {
            showToolTip = false
        }
    }

    MouseArea {
        id: mouseRegion
        anchors.fill: parent;
        property variant clickPos: "1,1"

        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onPressed: {
            if (mouse.button === Qt.RightButton) {
                Qt.callLater(Qt.quit)
            }
            clickPos = Qt.point(mouse.x,mouse.y)
        }

        onPositionChanged: {
            var delta = Qt.point(mouse.x-clickPos.x, mouse.y-clickPos.y)
            mainWindow.x += delta.x
            mainWindow.y += delta.y
        }
    }
}
