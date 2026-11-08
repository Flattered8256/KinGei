# KinGei

KinGei 是完全开源的高性能即时通讯软件，采用 Rust 编写。支持好友、私聊、群聊与文件附件以及站点管理，可 Docker 自托管。运行时内存约 20MB，这允许你能部署到大多数linux边缘开发板以及免费托管到云平台。

仓库：https://github.com/Flattered8256/KinGei


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

### 存储方式

- **local（默认）**：文件写在 `STORAGE_LOCAL_DIR`，Docker 下是 `KinGei-data/files`；不需要额外进程。
- **s3**：使用 S3 兼容对象存储。仓库自带的 `docker-compose.s3.yml` 会启动单机 Garage 并自动建桶；接入外部 S3/MinIO 时请先创建 `S3_BUCKET`，`object_store` 不会自动建桶（`S3_MANAGE_BUCKET` 已移除）。启动时会校验 bucket，凭据需要 `ListBucket` 权限。
- 切换后端不会迁移已有文件。需要迁移时停服后按 object key 把对象同步到目标位置；local 下 key 就是相对路径。
- 升级提示：默认后端已从 S3 改为 local。已有 Garage 数据的部署请继续使用 S3 叠加命令（或设置 `STORAGE_DRIVER=s3`），否则旧文件不会出现在本地目录。

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
