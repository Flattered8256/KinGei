# KinGei

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[KinGei](https://flattered8256.github.io/KinGei/) 是完全开源的高性能即时通讯软件，采用 Rust 编写。支持好友、私聊、群聊与文件附件以及站点管理，可 Docker 自托管。运行时内存约 20MB，这允许你能部署到大多数linux边缘开发板以及免费托管到云平台。

## 部署


### 1. 克隆仓库

```bash
git clone https://github.com/Flattered8256/KinGei.git
cd KinGei
```

### 2. 准备环境变量

```bash
cp .env.example .env
```

编辑：

| 变量 | 说明 |
| --- | --- |
| `PORT` | 端口，默认 `3000` |

文件存储只由后端访问，不需要对外暴露域名；上传和下载都由后端中转。

### 3. 启动

```bash
# 本地磁盘存储（默认，不需要 Garage）
docker compose up -d --build

# S3 / Garage
docker compose -f docker-compose.yml -f docker-compose.s3.yml up -d --build

# PostgreSQL + 本地磁盘
docker compose -f docker-compose.yml -f docker-compose.postgres.yml up -d --build

# PostgreSQL + S3 / Garage
docker compose -f docker-compose.yml -f docker-compose.s3.yml -f docker-compose.postgres.yml up -d --build
```

### 4. 打开站点

浏览器访问 http://127.0.0.1:3000 。

打开注册页，创建第一个账号，默认第一个注册的账号为管理员账号。



### 更新

```bash
git pull
docker compose up -d --build
```

### 备份


备份或迁移时带走整个 `KinGei-data/`；如果 `STORAGE_LOCAL_DIR` 不在其中，也要一并带走。

### 常用命令

```bash
# 日志
docker compose logs -f kingei-app

# 停止
docker compose stop

# 再启动（不重新构建）
docker compose up -d

# 停掉并删除容器（数据目录仍在）
docker compose down
```
