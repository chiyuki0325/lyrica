import QtQuick
import QtQuick.Controls
import QtQuick.Layouts 1.12
import org.kde.kirigami as Kirigami

Kirigami.FormLayout {
    id: page

    property int cfg_tlyricMode: 0
    property alias cfg_verbose: verbose.checked
    property alias cfg_disabledPlayers: disabledPlayers.text
    property alias cfg_enabledLyricProviders: enabledLyricProviders.text
    property alias disabledFolders: disabledFolders.text

    Label {
        text: i18n('Note that the backend settings will share among all the Lyrica widgets.\nUsing only one widget is recommended.')
        font.bold: true
    }
    Label {
        text: i18n('After saving settings, right-click the widget\nthen select "Reload" to make the changes take effect.\n')
        font.bold: true
    }

    Label {
        text: i18n('Lyric translation mode:')
    }

    ComboBox {
        id: tlyricMode
        textRole: 'label'
        model: [
            {
                'label': i18n('Show original lyric only'),
                'value': 0
            },
            {
                'label': i18n('Show translation only'),
                'value': 1
            },
            {
                'label': i18n('Original lyric | translation'),
                'value': 2
            },
            {
                'label': i18n('Translation | original lyric'),
                'value': 3
            }
        ]
        onCurrentIndexChanged: cfg_tlyricMode = model[currentIndex]['value']

        Component.onCompleted: {
            for (var i = 0; i < model.length; i++) {
                if (model[i]['value'] == plasmoid.configuration.tlyricMode) {
                    tlyricMode.currentIndex = i
                }
            }
        }

        property string currentVal: model[currentIndex]['value']
    }

    CheckBox {
        id: verbose
        text: i18n("Show detailed logs in the journal")
    }

    TextField {
        id: disabledPlayers
        Kirigami.FormData.label: i18n("Disabled players (comma separated):")
        placeholderText: i18n("firefox,chromium,plasma-browser-integration,kdeconnect")
    }

    TextField {
        id: enabledLyricProviders
        Kirigami.FormData.label: i18n("Enabled lyric providers (comma separated):")
        placeholderText: i18n("mpris2_text,file,yesplaymusic,feeluown_netease,netease")
    }

    Label {
        text: i18n('(<html>For available providers, see the project\'s <a href="https://github.com/chiyuki0325/lyrica/blob/next/docs/LYRIC_PROVIDERS.md">GitHub page.</a></html>)')
        onLinkActivated: Qt.openUrlExternally(link)
    }

    Label {
        text: i18n('Online lyric search pattern:')
    }

    ComboBox {
        id: onlineSearchPattern
        textRole: 'label'
        model: [
            {
                'label': i18n('Title + Artist')
                'value': 0
            },
            {
                'label': i18n('Title only (may not accurate)'),
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

    TextArea {
        id: disabledFolders
        Kirigami.FormData.label: i18n("Disabled folders (one per line):\nMusics in these folders will be treated as instrumental and won't be searched for lyrics.")
        placeholderText: i18n("/home/user/Music/lyric\n/home/user/Music/lyric2")
    }

}
