---
name: rhyme-checker
description: Chinese poetry rhyme and meter analysis tool. Use this skill to query character rhyme groups, ci-pai (词牌) meter patterns, and validate poetry against traditional Chinese prosody rules using the rhyme-checker CLI tool.
---

# 格律检测工具技能

这个技能使用 `rhyme-checker` 命令行工具来检查中文诗词的格律、查询汉字韵部信息和词牌格律。重要：LLM Agent 需要使用 `--no-color` 选项以使结果不依赖于颜色显示。

## 工具路径

```bash
./target/debug/rhyme-checker
```

## 主要功能

### 1. 查询汉字韵部信息 (query-char-rhyme)

查询单个汉字的韵部、声调信息,并可选择显示该韵部的所有汉字。

**用法:**
```bash
./target/debug/rhyme-checker query-char-rhyme <汉字> [选项]
```

**参数:**
- `<汉字>`: 要查询的单个汉字

**选项:**
- `-s, --show-all`: 显示该韵部的所有汉字(每行显示20个字)

**示例:**
```bash
# 查询"春"字的韵部信息
./target/debug/rhyme-checker query-char-rhyme 春

# 查询"春"字并显示同韵部所有汉字
./target/debug/rhyme-checker query-char-rhyme 春 -s
```

**输出示例:**
```
韵部: 十一真, 平
```

### 2. 查询词牌信息 (query-ci-pai)

查询词牌的格律信息,包括词牌名、别名、字数、韵脚要求等。

**用法:**
```bash
./target/debug/rhyme-checker --no-color query-ci-pai --ci-pai <词牌名> [选项]
```

**参数:**
- `-c, --ci-pai <词牌名>`: 要查询的词牌名(必填)

**选项:**
- `-v, --variant <变体>`: 格律变种(如"定格"、"格一"等)。如为空则显示所有格律变种

**示例:**
```bash
# 查询"如梦令"的所有格律变种
./target/debug/rhyme-checker --no-color query-ci-pai --ci-pai 如梦令

# 查询"如梦令"的"定格"变种
./target/debug/rhyme-checker --no-color query-ci-pai --ci-pai 如梦令 --variant 定格
```

**输出示例:**
```
词牌名:如梦令
别名:忆仙姿、宴桃源
变体:定格
说明:又名《忆仙姿》、《宴桃源》。五代时后唐庄宗(李存勗)创作。
格律:
--- 中仄中平平仄
--- 中仄中平平仄
--- 中仄仄平平
--- 中仄仄平平仄
--- 平仄
--- 平仄
--- 中仄仄平平仄
```

### 3. 检查格律 (match-ci-pai)

检查给定文本是否符合指定词牌的格律要求。

**用法:**
```bash
./target/debug/rhyme-checker --no-color match-ci-pai --ci-pai <词牌名> --variant <变体> <文本>
```

**参数:**
- `-c, --ci-pai <词牌名>`: 词牌名(必填)
- `-v, --variant <变体>`: 格律变种,如"定格"、"格一"等(必填)
- `<文本>`: 要检查的词文本

**示例:**
```bash
./target/debug/rhyme-checker --no-color match-ci-pai --ci-pai 如梦令 --variant 定格 "常记溪亭日暮,沉醉不知归路。兴尽晚回舟,误入藕花深处。争渡,争渡,惊起一滩鸥鹭。"
```

## 全局选项

工具支持以下全局选项(在子命令之前使用):

- `-d, --data-dir <路径>`: 数据文件夹路径(默认: `data`)
- `-t, --dict-type <类型>`: 韵书类型
  - `pingshui`: 平水韵
  - `cilin`: 词林正韵（默认）

**使用不同韵书的示例:**
```bash
# 使用词林正韵查询
./target/debug/rhyme-checker -t cilin query-char-rhyme 春

# 指定自定义数据目录
./target/debug/rhyme-checker -d /path/to/data query-char-rhyme 春
```

## 格律符号说明

在词牌格律中使用的符号:
- `平`: 平声字
- `仄`: 仄声字
- `中`: 可平可仄
- `---`: 句子前缀

## 技能使用场景

1. **诗词创作辅助**:查询词牌格律要求,帮助用户按格律填词
2. **韵脚检查**:查询汉字的韵部,确保诗词押韵正确
3. **格律验证**:检查已完成的诗词作品是否符合格律要求
4. **学习工具**:帮助用户学习和理解中国古典诗词的格律规则

## 注意事项

1. 查询汉字时只能输入单个汉字
2. 词牌名查询支持模糊匹配(包含关系)
3. 检查格律时必须指定确切的词牌名和变体
4. 默认使用词林正韵,如需使用平水韵需要通过 `-t` 参数指定
5. 数据文件应放在 `data` 目录下(或通过 `-d` 参数指定的目录)

