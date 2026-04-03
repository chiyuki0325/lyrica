#!/bin/sh
# Version: 8 (Updated for Plasma 6 / metadata.json)

# This script will convert the *.po files to *.mo files, rebuilding the package/contents/locale folder.
# Feature discussion: https://phabricator.kde.org/D5209
# Eg: contents/locale/fr_CA/LC_MESSAGES/plasma_applet_ink.chyk.lyricakde.mo

DIR=`cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd`
metadataFile="$DIR/../metadata.json"

### Colors
TC_Red='\033[31m'; TC_Orange='\033[33m';
TC_LightGray='\033[90m'; TC_LightRed='\033[91m'; TC_LightGreen='\033[92m'; TC_Yellow='\033[93m'; TC_LightBlue='\033[94m';
TC_Reset='\033[0m'; TC_Bold='\033[1m';
if [ ! -t 1 ]; then
	TC_Red=''; TC_Orange='';
	TC_LightGray=''; TC_LightRed=''; TC_LightGreen=''; TC_Yellow=''; TC_LightBlue='';
	TC_Bold=''; TC_Reset='';
fi
function echoTC() {
	text="$1"
	textColor="$2"
	echo -e "${textColor}${text}${TC_Reset}"
}
function echoGray { echoTC "$1" "$TC_LightGray"; }
function echoRed { echoTC "$1" "$TC_Red"; }
function echoGreen { echoTC "$1" "$TC_LightGreen"; }

#--- 依赖检查 ---
if [ -z "$(which jq)" ]; then
	echoRed "[translate/build] Error: jq command not found. Need to install jq"
	echoRed "[translate/build] Running ${TC_Bold}'sudo apt install jq'"
	sudo apt install jq
fi

if [ -z "$(which msgfmt)" ]; then
	echoRed "[translate/build] Error: msgfmt command not found. Need to install gettext"
	echoRed "[translate/build] Running ${TC_Bold}'sudo apt install gettext'"
	sudo apt install gettext
fi

#--- 解析 metadata.json ---
if [ ! -f "$metadataFile" ]; then
	echoRed "[translate/build] Error: metadata.json not found at $metadataFile"
	exit 1
fi

plasmoidName=$(jq -r '.KPlugin.Id' "$metadataFile")

if [ "$plasmoidName" == "null" ] || [ -z "$plasmoidName" ]; then
	echoRed "[translate/build] Error: Couldn't read .KPlugin.Id from metadata.json"
	exit 1
fi

packageRoot="${DIR}/.." # Root of translatable sources
projectName="plasma_applet_${plasmoidName}" # project name

#--- 编译消息 ---
echoGray "[translate/build] Compiling messages"

function relativePath() {
	basePath=`realpath -- "$1"`
	longerPath=`realpath -- "$2"`
	echo "${longerPath#${basePath}*}"
}

catalogs=`find . -name '*.po' | sort`
for cat in $catalogs; do
	catLocale=`basename ${cat%.*}`
	moFilename="${catLocale}.mo"

	# 关键修复：添加 -m 参数，允许解析不存在的路径
	installPath="${packageRoot}/contents/locale/${catLocale}/LC_MESSAGES/${projectName}.mo"
	installPath=`realpath -m -- "$installPath"`

	relativeInstallPath=`relativePath "${packageRoot}" "${installPath}"`
	relativeInstallPath="${relativeInstallPath#/*}"

	echoGray "[translate/build] Converting '${cat}' => '${relativeInstallPath}'"

	msgfmt -o "${moFilename}" "${cat}"
	# 确保父目录存在
	mkdir -p "$(dirname "$installPath")"
	mv "${moFilename}" "${installPath}"
done

echoGreen "[translate/build] Done building messages"

#--- 测试与重启 ---
if [ "$1" = "--restartplasma" ]; then
	echo "[translate/build] ${TC_Bold}Restarting plasmashell${TC_Reset}"
	killall plasmashell
	# 在 Plasma 6 中，通常直接调用 plasmashell 即可，kstart5 已过时
	if [ -x "$(which kstart)" ]; then
		kstart plasmashell
	else
		plasmashell &
	fi
	echo "[translate/build] Done restarting plasmashell"
else
	echo "[translate/build] (re)install the plasmoid and restart plasmashell to test translations."
fi
