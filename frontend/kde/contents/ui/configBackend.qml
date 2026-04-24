import QtQuick
import QtQuick.Controls
import QtQuick.Layouts 1.12
import org.kde.kirigami as Kirigami

Kirigami.FormLayout {
    id: page

    property alias cfg_disabledPlayers: disabledPlayers.text
    property alias cfg_enabledLyricProviders: enabledLyricProviders.text
    property alias cfg_disabledFolders: disabledFolders.text
    property alias cfg_lyricSearchFolder: lyricSearchFolder.text

    property int cfg_onlineSearchPattern: 0
    property alias cfg_onlineSearchTimeout: onlineSearchTimeout.text
    property alias cfg_onlineSearchRetry: onlineSearchRetry.checked
    property alias cfg_onlineSearchMaxRetries: onlineSearchMaxRetries.text

    property alias cfg_lyricCacheEnabled: lyricCacheEnabled.checked
    property int cfg_lyricCacheTtlDays: 30

    Label {
        text: i18n("Note that the backend settings will share among all the Lyrica widgets.\nUsing only one widget is recommended.")
        font.bold: true
    }
    Label {
        text: i18n("After saving settings, right-click the widget\nthen select \"Reload\" to make the changes take effect.\n")
        font.bold: true
    }


    TextField {
        id: disabledPlayers
        Kirigami.FormData.label: i18n("Disabled players (comma separated):")
        placeholderText: "firefox,chromium,plasma-browser-integration,kdeconnect"
    }

    TextField {
        id: enabledLyricProviders
        Kirigami.FormData.label: i18n("Enabled lyric providers (comma separated):")
        placeholderText: "Mpris2Text,File,YesPlayMusic,SPlayer,NeteaseTrackID,FeelUOwnNetease,Netease"
    }

    Label {
        text: i18n("(<html>For available providers, see the project's <a href=\"https://github.com/chiyuki0325/lyrica/blob/v1/docs/LYRIC_PROVIDERS.md\">GitHub page.</a></html>)")
        onLinkActivated: Qt.openUrlExternally(link)
    }

    Item {
        Layout.fillWidth: true
    }

    Label {
        text: i18n("Online lyric search pattern:")
    }

    ComboBox {
        id: onlineSearchPattern
        textRole: 'label'
        model: [
            {
                'label': i18n("Title + Artist"),
                'value': 0
            },
            {
                'label': i18n("Title only (may not accurate)"),
                'value': 1
            }
        ]
        onCurrentIndexChanged: cfg_onlineSearchPattern = model[currentIndex]['value']

        Component.onCompleted: {
            for (var i = 0; i < model.length; i++) {
                if (model[i]['value'] == plasmoid.configuration.onlineSearchPattern) {
                    onlineSearchPattern.currentIndex = i
                }
            }
        }

        property string currentVal: model[currentIndex]['value']
    }

    TextField {
        id: onlineSearchTimeout
        Kirigami.FormData.label: i18n("Online search timeout (seconds):")
        placeholderText: "10"
        validator: IntValidator {bottom: 0; top: 500}
    }

    CheckBox {
        id: onlineSearchRetry
        text: i18n("Retry online search if failed")
    }

    TextField {
        id: onlineSearchMaxRetries
        Kirigami.FormData.label: i18n("Max retries for online search (if retry enabled):")
        placeholderText: "3"
        validator: IntValidator {bottom: 0; top: 100}
        visible: onlineSearchRetry.checked
    }

    CheckBox {
        id: lyricCacheEnabled
        text: i18n("Cache online lyrics to disk")
    }

    TextField {
        id: lyricCacheTtlDays
        Kirigami.FormData.label: i18n("Cache TTL (days, 0 = never expire):")
        placeholderText: "30"
        validator: IntValidator { bottom: 0; top: 3650 }
        visible: lyricCacheEnabled.checked
        text: cfg_lyricCacheTtlDays
        onTextChanged: if (text !== "") cfg_lyricCacheTtlDays = parseInt(text)
    }

    TextArea {
        id: disabledFolders
        Kirigami.FormData.label: i18n("Disabled folders (one per line):\nMusics in these folders will be treated as instrumental and won't be searched for lyrics.")
        placeholderText: "/home/user/Music/lyric\n/home/user/Music/lyric2"
    }

    TextArea {
        id: lyricSearchFolder
        Kirigami.FormData.label: i18n("Alternative folder to search for .lrc files:")
        placeholderText: "/home/user/Music/lrc"
    }

}
