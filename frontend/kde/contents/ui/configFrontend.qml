import QtQuick
import QtQuick.Controls
import QtQuick.Layouts 1.12
import org.kde.kirigami as Kirigami
import org.kde.kquickcontrols as KQuickControls

Kirigami.FormLayout {
    id: page

    property int cfg_tlyricMode: 0
    property alias cfg_characterLimit: characterLimit.text
    property alias cfg_shouldUseDefaultThemeFontSize: shouldUseDefaultThemeFontSize.checked
    property alias cfg_configuredFontSize: configuredFontSize.text
    property alias cfg_layoutHeight: layoutHeight.text
    property alias cfg_showReconnectingText: showReconnectingText.checked
    property alias cfg_shouldUseDefaultThemeTextColor: shouldUseDefaultThemeTextColor.checked
    property alias cfg_configuredTextColor: configuredTextColor.color

    ComboBox {
        Kirigami.FormData.label: i18n("Lyric translation mode:")
        id: tlyricMode
        textRole: 'label'
        model: [
            {
                'label': i18n("Show original lyric only"),
                'value': 0
            },
            {
                'label': i18n("Show translation only"),
                'value': 1
            },
            {
                'label': i18n("Original lyric | Translation"),
                'value': 2
            },
            {
                'label': i18n("Translation | Original lyric"),
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

    TextField {
        id: characterLimit
        Kirigami.FormData.label: i18n("Character Limit:")
        placeholderText: "50"
        validator: IntValidator {bottom: 0; top: 9999}
    }

    CheckBox {
        id: shouldUseDefaultThemeFontSize
        text: i18n("Ignore the setting below and use theme default size")
    }

    TextField {
        id: configuredFontSize
        Kirigami.FormData.label: i18n("Custom font size:")
        placeholderText: i18n("")
        validator: IntValidator {bottom: 0; top: 9999}
    }

    TextField {
        id: layoutHeight
        Kirigami.FormData.label: i18n("Layout Height:")
        placeholderText: i18n("")
        validator: IntValidator {bottom: 0; top: 9999}
    }

    CheckBox {
        id: showReconnectingText
        text: i18n("Show [Reconnecting...] text when connection lost")
    }

    CheckBox {
        id: shouldUseDefaultThemeTextColor
        text: i18n("Use theme default text color")
    }

    KQuickControls.ColorButton {
        id: configuredTextColor
        Kirigami.FormData.label: i18n("Custom text color:")
        enabled: !shouldUseDefaultThemeTextColor.checked
        showAlphaChannel: false
    }

}
