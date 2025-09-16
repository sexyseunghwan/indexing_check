# Indexing Check (색인 체크 프로그램)

Elasticsearch 인덱스의 색인 완료 상태를 모니터링하고 문제 발생 시 알림을 전송하는 Rust 기반 모니터링 시스템입니다.

## 프로젝트 개요

이 프로그램은 Elasticsearch 인덱스의 색인 작업이 예정된 시간에 정상적으로 완료되었는지 확인하고, 실패하거나 지연될 경우 이메일 및 텔레그램을 통해 알림을 전송합니다.

**작성자**: Seunghwan Shin
**생성일**: 2024-12-30
**현재 버전**: v2.2.0

## 주요 기능

- **스케줄 기반 모니터링**: CRON 표현식을 통한 인덱스별 개별 스케줄 관리
- **다중 알림 채널**: SMTP 이메일 및 텔레그램 봇을 통한 알림 전송
- **실시간 로그 분석**: Elasticsearch의 벡터 인덱싱 로그를 실시간으로 분석
- **유연한 알림 정책**:
  - 정적 색인: 실패 시 지속적 알림
  - 증분 색인: 실패 시 1회 알림
- **Connection Pool 관리**: Semaphore 기반 효율적인 Elasticsearch 연결 관리

## 시스템 아키텍처

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Main Handler      │    │   Query Service     │    │ Notification Service│
│                     │◄──►│                     │◄──►│                     │
│ - 스케줄 관리        │    │ - ES 쿼리 실행       │    │ - 이메일 발송        │
│ - 알림 로직 제어     │    │ - 로그 분석          │    │ - 텔레그램 발송      │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
           │                           │                           │
           │                           │                           │
           ▼                           ▼                           ▼
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│   Repository        │    │   Elasticsearch     │    │   External APIs     │
│                     │    │                     │    │                     │
│ - ES Repository     │    │ - 인덱스 로그        │    │ - SMTP Server       │
│ - SQL Repository    │    │ - 색인 상태 정보     │    │ - Telegram Bot API  │
│ - Telegram Repo     │    │                     │    │                     │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

## 버전 히스토리

### v2.2.0 (2025-09-13)
- 코드 리팩토링

### v2.1.0 (2025-09-11)
- 코드 리팩토링

### v2.0.0 (2025-08-05)
- 코드 구조 전면 개선
- SMTP → imailer 라이브러리 변경
- Elasticsearch 비밀번호 URL 인코딩 처리 추가

### v1.3.0 (2025-05-16)
- Elasticsearch connection Pool을 Mutex → Semaphore로 변경

### v1.2.0 (2025-04-22)
- 증분색인 실패 시 1회 알림 정책으로 변경

### v1.1.0 (2025-02-07)
- 색인 실패 시 지속적 알림 기능 추가
- .env 파일 기반 설정 관리 도입
- 알림 구조 형식 변경

### v1.0.0 (2024-12-30)
- 초기 버전 생성

## 설정 파일

### system_config.toml
시스템 전반적인 설정을 관리합니다.

```toml
[elasticsearch]
elastic_host = ["host1:port", "host2:port", "host3:port"]
elastic_id = "username"
elastic_pw = "password"
elastic_pool_cnt = 3
elastic_cluster_name = "cluster-name"

[smtp]
smtp_name = "smtp.gmail.com"
credential_id = "email@gmail.com"
credential_pw = "app_password"
async_process_yn = true

[telegram]
bot_token = "your_bot_token"
chat_room_id = "your_chat_id"

[system]
log_index_name = "vector-indexing-logs"
schedule_term = 1000
err_monitor_index = "elastic-monitor-index"
message_chunk_size = 5

[code_type]
code_type = "prod"  # or "dev"
```

### index_list.toml
모니터링할 인덱스들의 스케줄 정보를 정의합니다.

```toml
[[index]]
index_name = "your_index_name"
time = "0 40 17 * * * *"  # CRON 표현식
duration = 900             # 최대 실행 시간(초)
size = 80000              # 예상 문서 수
indexing_type = "static index"  # "static index" 또는 "dynamic index"
```

### email_receiver_info.toml
이메일 수신자 정보를 관리합니다.

## 프로젝트 구조

```
src/
├── main.rs                     # 메인 엔트리 포인트
├── common.rs                   # 공통 import 및 상수
├── prelude.rs                  # 전역 사용 타입 및 트레이트
├── external_deps.rs            # 외부 의존성 관리
├── env_configuration/          # 환경 설정 관리
│   ├── mod.rs
│   └── env_config.rs
├── model/                      # 데이터 모델
│   ├── mod.rs
│   ├── total_config.rs         # 전체 설정 구조체
│   ├── index_schedules_config.rs
│   ├── email_struct.rs
│   ├── error_alarm_info.rs
│   ├── vector_index_log.rs
│   └── ...
├── handler/                    # 비즈니스 로직 핸들러
│   ├── mod.rs
│   └── main_handler.rs         # 메인 스케줄링 로직
├── service/                    # 서비스 레이어
│   ├── mod.rs
│   ├── query_service.rs        # Elasticsearch 쿼리 서비스
│   ├── notification_service.rs # 알림 서비스
│   ├── smtp_service.rs         # SMTP 서비스
│   └── index_storage_service.rs
├── repository/                 # 데이터 액세스 레이어
│   ├── mod.rs
│   ├── es_repository.rs        # Elasticsearch 리포지토리
│   ├── sqlserver_repository.rs
│   └── telegram_repository.rs
├── traits/                     # 트레이트 정의
│   ├── mod.rs
│   ├── service_traits/
│   └── repository_traits/
└── utils_modules/              # 유틸리티 모듈
    ├── mod.rs
    ├── logger_utils.rs         # 로깅 유틸리티
    ├── time_utils.rs           # 시간 처리 유틸리티
    ├── io_utils.rs             # 파일 I/O 유틸리티
    └── traits.rs
```

## 설치 및 실행

### 요구사항
- Rust 1.70 이상
- Elasticsearch 클러스터 접근 권한
- SMTP 서버 계정 (Gmail 등)
- 텔레그램 봇 토큰 (선택사항)

### 설치
```bash
# 리포지토리 클론
git clone <repository_url>
cd indexing_check

# 의존성 설치 및 빌드
cargo build --release
```

### 설정
1. `.env` 파일 생성 및 경로 설정
2. `config/system_config.toml` 파일 설정
3. `config/index_list.toml` 파일에 모니터링할 인덱스 추가
4. `config/email_receiver_info.toml` 파일에 수신자 정보 추가

### 실행
```bash
# 프로덕션 모드로 실행
cargo run --release

# 또는 빌드된 바이너리 실행
./target/release/indexing_check
```

## 모니터링 로직

1. **스케줄 기반 실행**: 각 인덱스별로 설정된 CRON 스케줄에 따라 모니터링 수행
2. **로그 분석**: Elasticsearch의 `vector-indexing-logs` 인덱스에서 최근 로그 검색
3. **상태 판단**:
   - 성공: 예상 문서 수 달성 및 "worked" 상태 확인
   - 실패: 에러 로그 발견 또는 예상 시간 초과
4. **알림 발송**: 실패 시 설정된 채널로 알림 전송

## 주요 의존성

- **tokio**: 비동기 런타임
- **elasticsearch**: Elasticsearch 클라이언트
- **lettre**: SMTP 이메일 전송
- **reqwest**: HTTP 클라이언트 (텔레그램 API)
- **cron**: CRON 스케줄 파싱
- **serde**: 직렬화/역직렬화
- **anyhow**: 에러 처리
- **flexi_logger**: 로깅

## 라이센스

이 프로젝트는 내부 사용을 위한 것입니다.

## 문의

기술적 문의사항은 작성자에게 연락 바랍니다.