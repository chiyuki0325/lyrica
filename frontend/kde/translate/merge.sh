#!/bin/sh
# Version: 24 (Updated for Plasma 6 / metadata.json)

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
	echoRed "[translate/merge] Error: jq command not found. Need to install jq"
	echoRed "[translate/merge] Running ${TC_Bold}'sudo apt install jq'"
	sudo apt install jq
fi

if [ -z "$(which xgettext)" ]; then
	echoRed "[translate/merge] Error: xgettext command not found. Need to install gettext"
	echoRed "[translate/merge] Running ${TC_Bold}'sudo apt install gettext'"
	sudo apt install gettext
fi

#--- 解析 metadata.json ---
if [ ! -f "$metadataFile" ]; then
	echoRed "[translate/merge] Error: metadata.json not found at $metadataFile"
	exit 1
fi

# 使用 jq 提取信息
plasmoidName=$(jq -r '.KPlugin.Id' "$metadataFile")
widgetName=$(jq -r '.KPlugin.Name' "$metadataFile")
website=$(jq -r '.KPlugin.Website' "$metadataFile")

if [ "$plasmoidName" == "null" ] || [ -z "$plasmoidName" ]; then
	echoRed "[translate/merge] Error: Could not read .KPlugin.Id from metadata.json"
	exit 1
fi

bugAddress="$website"
packageRoot=".." # Root of translatable sources
projectName="plasma_applet_${plasmoidName}" # project name

#---
echoGray "[translate/merge] Extracting messages"
potArgs="--from-code=UTF-8 --width=200 --add-location=file"

# 提取 .desktop 或 .json 相关的元数据翻译 (此处保留原逻辑以兼容旧文件)
find "${packageRoot}" -name '*.desktop' | sort > "${DIR}/infiles.list"
xgettext \
	${potArgs} \
	--files-from="${DIR}/infiles.list" \
	--language=Desktop \
	-k -kName -kGenericName -kComment -kKeywords \
	-D "${packageRoot}" \
	-D "${DIR}" \
	-o "template.pot.new" \
	|| \
	{ echoRed "[translate/merge] error while calling xgettext. aborting."; exit 1; }

sed -i 's/"Content-Type: text\/plain; charset=CHARSET\\n"/"Content-Type: text\/plain; charset=UTF-8\\n"/' "template.pot.new"

# 提取源码中的消息
find "${packageRoot}" -name '*.cpp' -o -name '*.h' -o -name '*.c' -o -name '*.qml' -o -name '*.js' | sort > "${DIR}/infiles.list"
xgettext \
	${potArgs} \
	--files-from="${DIR}/infiles.list" \
	-C -kde \
	-ci18n \
	-ki18n:1 -ki18nc:1c,2 -ki18np:1,2 -ki18ncp:1c,2,3 \
	-kki18n:1 -kki18nc:1c,2 -kki18np:1,2 -kki18ncp:1c,2,3 \
	-kxi18n:1 -kxi18nc:1c,2 -kxi18np:1,2 -kxi18ncp:1c,2,3 \
	-kkxi18n:1 -kkxi18nc:1c,2 -kkxi18np:1,2 -kkxi18ncp:1c,2,3 \
	-kI18N_NOOP:1 -kI18NC_NOOP:1c,2 \
	-kI18N_NOOP2:1c,2 -kI18N_NOOP2_NOSTRIP:1c,2 \
	-ktr2i18n:1 -ktr2xi18n:1 \
	-kN_:1 \
	-kaliasLocale \
	--package-name="${widgetName}" \
	--msgid-bugs-address="${bugAddress}" \
	-D "${packageRoot}" \
	-D "${DIR}" \
	--join-existing \
	-o "template.pot.new" \
	|| \
	{ echoRed "[translate/merge] error while calling xgettext. aborting."; exit 1; }

sed -i 's/# SOME DESCRIPTIVE TITLE./'"# Translation of ${widgetName} in LANGUAGE"'/' "template.pot.new"
sed -i 's/# Copyright (C) YEAR THE PACKAGE'"'"'S COPYRIGHT HOLDER/'"# Copyright (C) $(date +%Y)"'/' "template.pot.new"

# 检查 POT 文件更新
if [ -f "template.pot" ]; then
	newPotDate=`grep "POT-Creation-Date:" template.pot.new | sed 's/.\{3\}$//'`
	oldPotDate=`grep "POT-Creation-Date:" template.pot | sed 's/.\{3\}$//'`
	sed -i 's/'"${newPotDate}"'/'"${oldPotDate}"'/' "template.pot.new"
	changes=`diff "template.pot" "template.pot.new"`
	if [ ! -z "$changes" ]; then
		sed -i 's/'"${oldPotDate}"'/'"${newPotDate}"'/' "template.pot.new"
		mv "template.pot.new" "template.pot"
		addedKeys=`echo "$changes" | grep "> msgid" | cut -c 9- | sort`
		removedKeys=`echo "$changes" | grep "< msgid" | cut -c 9- | sort`
		echo ""
		echoGreen "Added Keys:"
		echoGreen "$addedKeys"
		echo ""
		echoRed "Removed Keys:"
		echoRed "$removedKeys"
		echo ""
	else
		rm "template.pot.new"
	fi
else
	mv "template.pot.new" "template.pot"
fi

potMessageCount=`expr $(grep -Pzo 'msgstr ""\n(\n|$)' "template.pot" | grep -c 'msgstr ""')`
echo "|  Locale  |  Lines  | % Done|" > "./Status.md"
echo "|----------|---------|-------|" >> "./Status.md"
entryFormat="| %-8s | %7s | %5s |"
templateLine=`perl -e "printf(\"$entryFormat\", \"Template\", \"${potMessageCount}\", \"\")"`
echo "$templateLine" >> "./Status.md"

rm "${DIR}/infiles.list"
echoGray "[translate/merge] Done extracting messages"

#--- 合并 PO 文件 ---
echoGray "[translate/merge] Merging messages"
catalogs=`find . -name '*.po' | sort`
for cat in $catalogs; do
	echoGray "[translate/merge] Updating ${cat}"
	catLocale=`basename ${cat%.*}`
	widthArg=""
	catUsesGenerator=`grep "X-Generator:" "$cat"`
	if [ -z "$catUsesGenerator" ]; then
		widthArg="--width=400"
	fi

	compendiumArg=""
	if [ ! -z "$COMPENDIUM_DIR" ]; then
		langCode=`basename "${cat%.*}"`
		compendiumPath=`realpath "$COMPENDIUM_DIR/compendium-${langCode}.po"`
		if [ -f "$compendiumPath" ]; then
			compendiumArg="--compendium=$compendiumPath"
		fi
	fi

	cp "$cat" "$cat.new"
	sed -i 's/"Content-Type: text\/plain; charset=CHARSET\\n"/"Content-Type: text\/plain; charset=UTF-8\\n"/' "$cat.new"

	msgmerge \
		${widthArg} \
		--add-location=file \
		--no-fuzzy-matching \
		${compendiumArg} \
		-o "$cat.new" \
		"$cat.new" "${DIR}/template.pot"

	sed -i 's/# SOME DESCRIPTIVE TITLE./'"# Translation of ${widgetName} in ${catLocale}"'/' "$cat.new"
	sed -i 's/# Translation of '"${widgetName}"' in LANGUAGE/'"# Translation of ${widgetName} in ${catLocale}"'/' "$cat.new"
	sed -i 's/# Copyright (C) YEAR THE PACKAGE'"'"'S COPYRIGHT HOLDER/'"# Copyright (C) $(date +%Y)"'/' "$cat.new"

	poEmptyMessageCount=`expr $(grep -Pzo 'msgstr ""\n(\n|$)' "$cat.new" | grep -c 'msgstr ""')`
	poMessagesDoneCount=`expr $potMessageCount - $poEmptyMessageCount`
	poCompletion=`perl -e "printf(\"%d\", $poMessagesDoneCount * 100 / $potMessageCount)"`
	poLine=`perl -e "printf(\"$entryFormat\", \"$catLocale\", \"${poMessagesDoneCount}/${potMessageCount}\", \"${poCompletion}%\")"`
	echo "$poLine" >> "./Status.md"
	mv "$cat.new" "$cat"
done
echoGray "[translate/merge] Done merging messages"

#--- 更新 ReadMe.md ---
echoGray "[translate/merge] Updating translate/ReadMe.md"
sed -i -E 's`share\/plasma\/plasmoids\/(.+)\/translate`share/plasma/plasmoids/'"${plasmoidName}"'/translate`' ./ReadMe.md
if [[ "$website" == *"github.com"* ]]; then
	sed -i -E 's`\[new issue\]\(https:\/\/github\.com\/(.+)\/(.+)\/issues\/new\)`[new issue]('"${website}"'/issues/new)`' ./ReadMe.md
fi
sed -i '/^|/ d' ./ReadMe.md
cat ./Status.md >> ./ReadMe.md
rm ./Status.md

echoGreen "[translate/merge] Done merge script"
