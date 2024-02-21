import QtQuick 2.0
import QtQuick.Controls 2.5
import QtQuick.Layouts 1.12
import org.kde.kirigami 2.4 as Kirigami

Kirigami.FormLayout {
    id: page

    property alias cfg_characterLimit: characterLimit.text
    property alias cfg_shouldUseDefaultThemeFontSize: shouldUseDefaultThemeFontSize.checked
    property alias cfg_configuredFontSize: configuredFontSize.text

    TextField {
        id: characterLimit
        Kirigami.FormData.label: i18n("Character Limit:")
        placeholderText: i18n("")
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
}
