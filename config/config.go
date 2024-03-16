package config

import (
	"gopkg.in/yaml.v3"
	"os"
)

type ConfigType struct {
	Verbose               bool     `yaml:"verbose"`
	DisabledPlayers       []string `yaml:"disabled_players"`
	EnabledLyricProviders []string `yaml:"enabled_lyric_providers"`
	ProviderSettings      struct {
		NetEase struct {
			UseTranslateLyric bool `yaml:"use_translate_lyric"`
		} `yaml:"netease"`
		YesPlayMusic struct {
			UseTranslateLyric bool `yaml:"use_translate_lyric"`
		} `yaml:"yesplaymusic"`
	} `yaml:"provider_settings"`
	DisabledFolders []string `yaml:"disabled_folders"`
}

var ConfigDir string

var Config = ConfigType{
	Verbose: false,
	DisabledPlayers: []string{
		"firefox",
		"chromium",
		"plasma-browser-integration",
		"kdeconnect",
	},
	EnabledLyricProviders: []string{
		"file",
		"yesplaymusic",
		"netease",
	},
}

func LoadConfig() {
	// 获取配置文件目录
	ConfigDir = os.Getenv("XDG_CONFIG_HOME")
	if ConfigDir == "" {
		ConfigDir = os.Getenv("HOME") + "/.config"
	}

	configFilePath := ConfigDir + "/desktop-lyrics-kde/config.yml"
	if _, err := os.Stat(configFilePath); os.IsNotExist(err) {
		// 创建配置文件
		_ = os.MkdirAll(ConfigDir+"/desktop-lyrics-kde", 0755)
		_, _ = os.Create(configFilePath)
	}

	configContent, _ := os.ReadFile(configFilePath)

	_ = yaml.Unmarshal(configContent, &Config)
}
