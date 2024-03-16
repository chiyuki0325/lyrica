package config

import (
	"gopkg.in/yaml.v3"
	"os"
)

type ConfigType struct {
	Verbose               bool     `yaml:"verbose"`
	BlockedPlayers        []string `yaml:"blocked_players"`
	EnabledLyricProviders []string `yaml:"enabled_lyric_providers"`
}

var Config = ConfigType{
	Verbose: false,
	BlockedPlayers: []string{
		"firefox",
		"chromium",
		"plasma-browser-integration",
	},
	EnabledLyricProviders: []string{
		"file",
	},
}

func LoadConfig() {
	// 获取配置文件目录
	configDir := os.Getenv("XDG_CONFIG_HOME")
	if configDir == "" {
		configDir = os.Getenv("HOME") + "/.config"
	}

	configFilePath := configDir + "/desktop-lyrics-kde/config.yml"
	if _, err := os.Stat(configFilePath); os.IsNotExist(err) {
		// 创建配置文件
		_ = os.MkdirAll(configDir+"/desktop-lyrics-kde", 0755)
		_, _ = os.Create(configFilePath)
	}

	configContent, _ := os.ReadFile(configFilePath)

	_ = yaml.Unmarshal(configContent, &Config)
}
