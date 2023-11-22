快捷键自动生成文件头部注释，包含copyright和license信息，以符合开源社区规范
1、vscode中安装psioniq File Header插件
2、在settings.json中做如下配置

    "psi-header.config": {
		"forceToTop": true,
		"blankLinesAfter": 1,
		"spacesBetweenYears": false,
		"license": "CustomUri",  //表示引用定制的license文件内容
		"author": "RobustMQ Team",
		"authorEmail": "robustmq@outlook.com",
	},

    "psi-header.templates": [
		{
			"language": "rust",
			"template": [
				"Copyright (c) <<yeartoyear(fc, now)>> <<author>> ",
				"<<licensetext>>"  //license系统参数，用于引用license_header.file中的文本内容
			],
		},
	],

    "psi-header.license-reference": {
		"uri": "/Users/justinliu/Documents/workspace/robustmq/config/license_header.file", 
        //注意：这里需要填写license文件的本地绝对路径
		"uriIsLocalFile": true
	}
	
3、新建源码文件后，执行快捷键control + option + H 生成带license的注释文件头