itsp-todo-server
===

[![CircleCI](https://circleci.com/gh/ArkArk/itsp-todo-server.svg?style=svg)](https://circleci.com/gh/ArkArk/itsp-todo-server)

This is homework for [ITSP](http://www.itpro.titech.ac.jp/) (1Q).

## TL;DR

レポート課題：簡易な TODO 管理サービス用の HTTP サーバを作成する

- TODO管理用のAPI（登録・取得・削除）を提供する
- データの送受信にはJSONを使用する
- サーバ側のデータはデータベース上で保持する
- (言語, フレームワーク) : (Rust, actix-web)

## Technology

-  [:crab: Rust](https://www.rust-lang.org/)
-  [:gear: actix, actix-web](https://actix.rs/)
-  [:oil_drum: Diesel](http://diesel.rs/)
-  [:elephant: PostgreSQL](https://www.postgresql.org/)
-  [:whale: Docker](https://www.docker.com/)
-  [:octopus: docker-compose](https://github.com/docker/compose)
-  [:minidisc: CircleCI](https://circleci.com/)

## Usage

### Prerequisites

- Rust >= 1.35
- Docker
- docker-compose

### Setup the database

```sh
# Install diesel
$ cargo install diesel_cli --no-default-features --features postgres
# Create and start a DB container
$ docker-compose up -d
# Setup the DB
$ diesel setup
```

### Run the application
```sh
$ cargo run
```

### Specification

起動すると`8080`ポートをlistenする。

```sh
# イベント登録 API request
POST /api/v1/event
{"deadline": "2019-06-11T14:00:00+09:00", "title": "レポート提出", "memo": ""}

# イベント登録 API response
200 OK
{"status": "success", "message": "registered", "id": 1}

400 Bad Request
{"status": "failure", "message": "invalid date format"}
```

```sh
# イベント全取得 API request
GET /api/v1/event

# イベント全取得 API response
200 OK
{"events": [
    {"id": 1, "deadline": "2019-06-11T14:00:00+09:00", "title": "レポート提出", "memo": ""},
    ...
]}
```

```sh
# イベント1件取得 API request
GET /api/v1/event/${id}

# イベント1件取得 API response
200 OK
{"id": 1, "deadline": "2019-06-11T14:00:00+09:00", "title": "レポート提出", "memo": ""}

404 Not Found
```

```sh
# イベント1件削除 API request
DELETE /api/v1/event/${id}

# イベント1件削除 API response
200 OK
{"id": 1, "deadline": "2019-06-11T14:00:00+09:00", "title": "レポート提出", "memo": ""}

404 Not Found
```

### Example

[httpie](https://httpie.org/)を使用したコマンド例：

```sh
# 全イベントの取得
$ http GET localhost:8080/api/v1/event

# イベントを追加
#   { deadline=2019-06-11T14:00:00+09:00, title="test title", memo="test memo" }
$ http POST localhost:8080/api/v1/event deadline=2019-06-11T14:00:00+09:00 title="test title" memo="test memo"

# 1番目のイベントを取得
$ http GET localhost:8080/api/v1/event/1

# 1番目のイベントを削除
$ http DELETE localhost:8080/api/v1/event/1
```

## Local tests

ローカルでのテストは以下の手順で行う：

```sh
$ cd tests
$ docker-compose up -d
$ diesel setup
$ cargo test
$ docker-compose down
```
