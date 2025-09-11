# Indexing Check 🔍

Elasticsearch 색인 작업 완료 여부를 모니터링하고 실패 시 알림을 발송하는 Rust 애플리케이션입니다.

## 📋 개요

이 프로그램은 Elasticsearch의 벡터 색인 작업이 올바르게 완료되었는지 확인하고, 문제가 발생했을 때 자동으로 알림을 전송합니다. 스케줄링 기반으로 동작하며 다중 인덱스를 동시에 모니터링할 수 있습니다.

## ✨ 주요 기능

- **자동 스케줄링**: CRON 표현식을 사용한 유연한 스케줄링
- **다중 인덱스 모니터링**: 여러 인덱스를 동시에 모니터링
- **실시간 알림**: Telegram 및 이메일을 통한 즉시 알림
- **오류 분류**: 완전 실패와 부분 실패를 구분하여 처리
- **로그 관리**: 구조화된 로깅 시스템
- **안전한 비동기 처리**: Tokio 기반 고성능 비동기 처리

## 🛠️ 기술 스택

- **언어**: Rust (Edition 2021)
- **비동기 런타임**: Tokio
- **데이터베이스**: Elasticsearch, SQL Server
- **알림**: Telegram Bot API, SMTP 이메일
- **설정 관리**: TOML 파일 기반
- **로깅**: flexi_logger

## 📁 프로젝트 구조

```
indexing_check/
├── src/
│   ├── handler/           # 메인 핸들러 로직
│   ├── service/           # 비즈니스 로직 서비스
│   ├── repository/        # 데이터 액세스 계층
│   ├── model/             # 데이터 모델
│   ├── traits/            # 트레이트 정의
│   ├── utils_modules/     # 유틸리티 함수
│   └── env_configuration/ # 환경 설정
├── config/                # 설정 파일
├── logs/                  # 로그 파일
├── html/                  # HTML 템플릿
└── target/                # 빌드 결과물
```

## 🚀 설치 및 실행

### 전제 조건

- Rust 1.70+ 설치
- Elasticsearch 클러스터 접근 권한
- SMTP 서버 또는 Telegram Bot 토큰

### 설치

```bash
# 저장소 클론
git clone <repository-url>
cd indexing_check

# 의존성 설치 및 빌드
cargo build --release
```

### 설정

1. **환경 변수 설정** (`.env` 파일):
```env
INDEX_LIST_PATH="./config/index_list.toml"
EMAIL_RECEIVER_PATH="./config/email_receiver_info.toml"
SYSTEM_CONFIG_PATH="./config/system_config.toml"
HTML_TEMPLATE_PATH="./html/view.html"
SQL_SERVER_INFO_PATH="./config/sql_server_info.toml"
```

2. **인덱스 목록 설정** (`config/index_list.toml`):
```toml
[[index]]
index_name = "your-index-name"
time = "0 30 2 * * *"  # CRON 표현식
duration = 3600        # 확인 기간 (초)
size = 1000           # 최소 문서 수
indexing_type = "full index"
```

3. **시스템 설정** (`config/system_config.toml`):
```toml
schedule_term = 30000
log_index_name = "vector-indexing-logs"
err_monitor_index = "indexing-error-monitor"
```

### 실행

```bash
# 프로덕션 실행
cargo run --release

# 개발 모드 실행
cargo run
```

## 📊 모니터링 로직

### 1. 색인 상태 확인
- 지정된 시간 범위 내의 Elasticsearch 로그를 조회
- "index worked" 메시지 존재 여부 확인
- 색인된 문서 수 검증

### 2. 오류 분류
- **Full Error**: 색인 작업이 전혀 실행되지 않음
- **Partial Error**: 색인은 실행되었으나 문서 수가 기준 미달

### 3. 알림 처리
- 오류 발생 시 Elasticsearch 오류 인덱스에 기록
- Telegram 및 이메일로 즉시 알림 발송
- 증분 색인의 경우 일회성 알림 후 자동 삭제

## 🔧 주요 컴포넌트

### MainHandler
- 스케줄링 및 모니터링 로직의 중심
- 비동기 작업 조율 및 오류 처리

### QueryService
- Elasticsearch 및 SQL Server 연동
- 데이터 조회 및 저장 담당

### NotificationService
- Telegram 및 이메일 알림 발송
- 다중 수신자 지원

## 📝 로깅

- **위치**: `./logs/` 디렉토리
- **형식**: 구조화된 로그 (시간, 레벨, 메시지)
- **레벨**: ERROR, WARN, INFO, DEBUG

## 🔒 보안 고려사항

- 환경 변수를 통한 민감정보 관리
- TOML 파일의 적절한 권한 설정 필요
- Elasticsearch 및 SMTP 연결 보안

## 📅 버전 히스토리

- **v2.1.0** (2025-09-11): 코드 리팩토링
- **v2.0.0** (2025-08-05): 구조 개선 및 SMTP → imailer 전환
- **v1.3.0** (2025-05-16): Connection Pool 최적화
- **v1.2.0** (2025-04-22): 증분색인 알림 로직 개선
- **v1.1.0** (2025-02-07): 알림 구조 개선
- **v1.0.0** (2024-12-30): 초기 버전

## 👨‍💻 개발자

**Seunghwan Shin**
- 최초 개발: 2024-12-30
- 지속적인 개선 및 유지보수

## 📄 라이선스

이 프로젝트는 내부 사용을 위한 프로그램입니다.

## 🤝 기여

버그 리포트나 기능 제안은 이슈 트래커를 통해 제출해 주세요.

---

> **참고**: 이 프로그램은 운영 환경에서 중요한 데이터 색인 작업을 모니터링하므로, 설정 변경 시 충분한 테스트를 거친 후 적용하시기 바랍니다.