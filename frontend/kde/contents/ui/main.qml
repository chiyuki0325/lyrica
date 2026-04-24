import QtQuick 2.15
import QtQuick.Layouts 1.1
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.plasma5support as Plasma5Support
import org.kde.kirigami as Kirigami
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
        property bool showPlaceholder: true


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
            url: "ws://127.0.0.1:15650/ws"
            onTextMessageReceived: (message) => {
                message = JSON.parse(message)
                switch (message["id"]) {
                    case 1:
                        // Update music metadata
                        text.text = ""
                        oneLineLayout.showPlaceholder = true
                        oneLineLayout.updateLayoutSize()
                        break
                    case 0:
                        // Update lyric line
                        let lyric_text = message["data"]["update_lyric_line"]["text"]
                        let lyric_alt = message["data"]["update_lyric_line"]["alt"]
                        let update_line = i18n("[No lyric]")
                        switch (plasmoid.configuration.tlyricMode) {
                            case 0:
                                update_line = lyric_text || ""
                                break
                            case 1:
                                update_line = lyric_alt || lyric_text || ""
                                break
                            case 2:
                                update_line = lyric_text || ""
                                if (lyric_alt) {
                                    update_line += " | " + lyric_alt
                                }
                                break
                            case 3:
                                if (lyric_alt) {
                                    update_line = lyric_alt + " | " + lyric_text
                                } else {
                                    update_line = lyric_text || ""
                                }
                                break
                        }
                        if (update_line.length > plasmoid.configuration.characterLimit) {
                            update_line = update_line.slice(0, plasmoid.configuration.characterLimit) + "..."
                        }
                        text.text = update_line
                        oneLineLayout.showPlaceholder = (update_line.length == 0)
                        oneLineLayout.updateLayoutSize()
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
                    oneLineLayout.updateLayoutSize()
                    socket.active = false
                    delay(500, () => {
                        if (socket.active == false) {
                            socket.active = true
                        }
                    })
                } else if (status == WebSocket.Open) {
                    // Send config
                    const configString = JSON.stringify({
                        disabled_players: plasmoid.configuration.disabledPlayers.split(","),
                        enabled_lyric_providers: plasmoid.configuration.enabledLyricProviders.split(","),
                        online_search_pattern: plasmoid.configuration.onlineSearchPattern,
                        disabled_folders: plasmoid.configuration.disabledFolders.split("\n"),
                        online_search_timeout_secs: plasmoid.configuration.onlineSearchTimeout,
                        online_search_retry: plasmoid.configuration.onlineSearchRetry,
                        online_search_max_retries: plasmoid.configuration.onlineSearchMaxRetries,
                        lyric_search_folder: plasmoid.configuration.lyricSearchFolder || "~/Music/lrc",
                        lyric_cache_enabled: plasmoid.configuration.lyricCacheEnabled,
                        lyric_cache_ttl_days: plasmoid.configuration.lyricCacheTtlDays,
                    })
                    const xhr = new XMLHttpRequest()
                    console.log("[lyrica] Updating config")
                    xhr.open("POST", "http://127.0.0.1:15650/config/update", true)
                    xhr.setRequestHeader("Content-Type", "application/json")
                    xhr.onreadystatechange = () => {
                        if (xhr.readyState == 4) {
                            console.log("[lyrica]" + xhr.responseText)
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
            color: plasmoid.configuration.shouldUseDefaultThemeTextColor
                 ? PlasmaCore.Theme.textColor
                 : plasmoid.configuration.configuredTextColor
        }


        Kirigami.Icon {
            source: plasmoid.configuration.placeholderIconName
            height: plasmoid.configuration.layoutHeight * 0.8
            width: height
            anchors.verticalCenter: parent.verticalCenter
            visible: oneLineLayout.showPlaceholder && plasmoid.configuration.placeholderIconName.length > 0
        }

        Plasmoid.contextualActions: [
            PlasmaCore.Action {
                text: i18n("Reload configuration")
                icon.name: "view-refresh-symbolic"
                priority: PlasmaCore.Action.LowPriority
                onTriggered: {
                    socket.active = false
                    delay(200, () => {
                        socket.active = true
                    })
                }
            },

            PlasmaCore.Action {
                text: i18n("Restart Lyrica")
                icon.name: "collapse-all-symbolic"
                priority: PlasmaCore.Action.LowPriority
                onTriggered: {
                    backendExecutable.disconnectSource(backendExecutable.command)
                    commandLine.connectSource("bash -c 'killall lyrica --signal=SIGKILL'")
                    delay(100, () => {
                        backendExecutable.connectSource(backendExecutable.command)
                    })
                }
            }
        ]


	    Plasma5Support.DataSource {
	        id: backendExecutable
	        readonly property string command: "bash -c '$HOME/.local/share/plasma/plasmoids/ink.chyk.lyricakde/contents/bin/lyrica'"
		    engine: "executable"
		    connectedSources: []
		    onSourceConnected: {
		        socket.active = true
		    }
	    }

	    Plasma5Support.DataSource {
	        id: commandLine
		    engine: "executable"
		    connectedSources: []
	    }

		Component.onCompleted: {
            backendExecutable.connectSource(backendExecutable.command)
		}
    }
}
