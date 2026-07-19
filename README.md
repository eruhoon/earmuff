# Earmuff (이어머프) 🎧

> **윈도우 내 특정 게임 및 프로그램 사운드를 항상 위에 띄워두고 원클릭으로 토글하는 미니멀 플로팅 사운드 위젯**

`Earmuff`는 백그라운드에서 실행 중인 게임(예: 로스트아크)이나 브라우저, 디스코드 등 특정 프로그램의 소리를 네이티브 윈도우 API를 통해 클릭 한 번으로 즉시 음소거/해제할 수 있는 아주 가볍고 세련된 데스크톱 플로팅 위젯입니다.

Tauri v2, Svelte 5, 그리고 Rust 기반의 Windows Core Audio API를 활용하여 외부 프로그램이나 VBScript 없이 순수 네이티브 방식으로 작동합니다.

---

## ✨ 핵심 기능 (Core Features)

1. **미니멀 플로팅 캡슐 위젯 UI (Glassmorphism)**
   - 윈도우 테두리가 없고 배경이 투명한 초소형 캡슐 형태의 디자인입니다.
   - **'화면 항상 위(Always on Top)'**로 고정되어 있어 게임 중에도 가려지지 않습니다.
   - 드롭다운이 없는 고정 사이즈(`160x60px`)로 은은한 그림자(Box Shadow) 효과가 가려짐 없이 깔끔하게 표현됩니다.

2. **드래그앤드롭 화면 이동 & 숨김 처리 (Skip Taskbar & Drag Handle)**
   - 위젯 좌측의 그립 영역(`::`)을 마우스로 잡고 화면 원하는 곳 어디로든 자유롭게 배치할 수 있습니다.
   - 위젯 성격에 맞추어 작업표시줄에는 아이콘이 나타나지 않으며, 오직 화면 위와 시스템 트레이 영역에서만 조용히 자리 잡습니다.

3. **네이티브 실행 파일 아이콘 실시간 추출 (Dynamic Icon Extraction)**
   - 윈도우 OS의 Shell API 및 GDI를 활용하여 **제어 대상 프로그램의 실제 아이콘을 메모리 상에서 비트맵으로 직접 추출**하여 위젯 중앙에 상시 노출합니다.
   - 프로그램이 꺼져 있거나 예외 상황일 때는 사전에 준비된 수려한 벡터 로고(로아 금빛 날개 검, 크롬 지구본, 디스코드 등)로 자동 대체되는 이중 폴백(Fallback)이 적용됩니다.

4. **마우스 휠 스크롤 대상 프로그램 전환 (Frictionless Selector)**
   - 위젯 버튼 위에 마우스를 올린 뒤 **휠을 스크롤하는 것만으로 제어할 대상을 순식간에 전환**합니다.
   - 휠을 돌릴 때마다 중간의 아이콘이 해당 프로그램의 실제 아이콘으로 실시간 동기화됩니다.
   - 시스템에서 소리가 나고 있는 활성 사운드 프로세스 목록만 추려내어 자동으로 옵션으로 제공합니다.

5. **2중 안전 종료 및 전체 소리 복구 시스템 (Context Menu)**
   - **작업 표시줄 우측 트레이 아이콘 우클릭** 또는 **위젯 손잡이(`::`) 영역 마우스 우클릭**을 통해 2가지 위치에서 동일한 컨텍스트 메뉴를 지원합니다.
   - **`전체 음소거 해제`**: 위젯으로 인해 개별 차단되었던 모든 프로그램의 사운드를 한 번에 원래대로 되돌립니다.
   - **`종료`**: 위젯 어플리케이션을 즉각 안전하게 종료합니다.

---

## 🛠️ 기술 스택 (Tech Stack)

- **Frontend**: Svelte 5 (Runes `$state`, `$effect`, Snippets), SvelteKit, TypeScript, Vanilla CSS (Glassmorphic Neumorphism)
- **Backend/Core**: Rust (Tauri v2), Windows Crate (0.61.3)
  - *Audio control*: `IMMDeviceEnumerator`, `IAudioSessionManager2`, `ISimpleAudioVolume` (COM Interface)
  - *Icon Extraction*: `SHGetFileInfoW` (Shell API), `GetDIBits`, `GetObjectW` (GDI Graphics)
  - *Process Monitoring*: `CreateToolhelp32Snapshot` (ToolHelp API)

---

## 🚀 개발 및 실행 방법 (Quick Start)

### 사전 요구사항 (Prerequisites)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/)
- [Rust & Cargo](https://www.rust-lang.org/) (Windows C++ 빌드 도구 포함)

### 패키지 설치 및 실행
```bash
# 1. 의존성 패키지 설치
pnpm install

# 2. 새로운 어플리케이션 로고 일괄 생성 (최초 1회 필수)
pnpm tauri icon app_icon_source.jpg

# 3. 개발 모드 빌드 및 실행
pnpm tauri dev
```

---

## 📝 라이선스 (License)

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
