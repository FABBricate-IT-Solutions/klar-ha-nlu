"""Lab / mapping / backup chrome for East and Southeast Asian Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["zh-CN", "zh-TW", "zh-HK", "ja", "ko", "th", "vi", "id", "ms", "mn"]

TABLE = """
parseHint	实验室是所选语言的 Assist 路径。这里的决定和意图由 Klar 执行。句子触发器仅在 Klar 不可达时运行。	實驗室是所選語言的 Assist 路徑。這裡的決定和意圖由 Klar 執行。句子觸發器僅在 Klar 不可達時執行。	實驗室係所選語言嘅 Assist 路徑。呢度嘅決定同意圖由 Klar 執行。句子觸發器淨係 Klar 唔到先跑。	ラボは選択した言語の Assist 経路です。ここの決定と意図は Klar が実行します。文トリガーは Klar に届かないときだけ動きます。	랩은 선택한 언어의 Assist 경로입니다. 여기 결정과 의도는 Klar가 실행합니다. 문장 트리거는 Klar에 닿지 않을 때만 실행됩니다.	แล็บคือเส้นทาง Assist ของภาษาที่เลือก การตัดสินใจและเจตนาที่นี่ Klar เป็นผู้รัน ทริกเกอร์ประโยคทำงานเมื่อ Klar ติดต่อไม่ได้เท่านั้น	Phòng thí nghiệm là đường Assist của ngôn ngữ đã chọn. Quyết định và ý định ở đây do Klar chạy. Bộ kích câu chỉ chạy khi Klar không tới được.	Lab adalah jalur Assist untuk bahasa yang dipilih. Keputusan dan niat di sini dijalankan Klar. Pemicu kalimat hanya jalan jika Klar tak terjangkau.	Makmal ialah laluan Assist untuk bahasa dipilih. Keputusan dan niat di sini dijalankan Klar. Pencetus ayat hanya jalan jika Klar tidak dapat dicapai.	Лаборатори нь сонгосон хэлний Assist зам. Эндхийн шийдвэр, зорилгыг Klar гүйцэтгэнэ. Өгүүлбэрийн триггер зөвхөн Klar хүрэхгүй үед ажиллана.
triggerFirst	Klar 解析，然后走这条路径。Assist 不会跑别的意图、句子触发器或天气回退。	Klar 解析，然後走這條路徑。Assist 不會跑別的意圖、句子觸發器或天氣後援。	Klar 解析，然後行呢條路。Assist 唔會跑第二個意圖、句子觸發器或者天氣後備。	Klar 解析のあと、この経路。Assist は別の意図、文トリガー、天候フォールバックを実行しません。	Klar 분석 후 이 경로. Assist는 다른 의도, 문장 트리거, 날씨 대체를 실행하지 않습니다.	แยก Klar แล้วตามเส้นทางนี้ Assist จะไม่รันเจตนาอื่น ทริกเกอร์ประโยค หรือสำรองอากาศ	Phân Klar, rồi đường đó. Assist không chạy ý định khác, bộ kích câu hay dự phòng thời tiết.	Uraian Klar, lalu jalur itu. Assist tidak menjalankan niat lain, pemicu kalimat, atau cadangan cuaca.	Huraian Klar, kemudian laluan itu. Assist tidak menjalankan niat lain, pencetus ayat, atau sandaran cuaca.	Klar задлал, дараа энэ зам. Assist өөр зорилго, өгүүлбэрийн триггер, цаг агаарын нөөц ажиллуулахгүй.
labPipeline	流水线	管線	管線	パイプライン	파이프라인	ท่อส่ง	Đường ống	Saluran	Saluran	Шугам
labChipContextOnly	仅上下文	僅上下文	淨上下文	文脈のみ	문맥만	บริบทเท่านั้น	chỉ ngữ cảnh	hanya konteks	hanya konteks	зөвхөн нөхцөл
labChipNluRag	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG
labChipSemantic	语义	語意	語意	意味	의미	ความหมาย	ngữ nghĩa	semantik	semantik	утга
labChipNoConfirm	无确认	無確認	無確認	確認なし	확인 없음	ไม่ยืนยัน	không xác nhận	tanpa konfirmasi	tanpa pengesahan	баталгаагүй
labChipLlmRefine	LLM 润色	LLM 潤飾	LLM 潤飾	LLM 洗練	LLM 다듬기	LLM ปรับ	tinh LLM	perhalus LLM	perhalusi LLM	LLM нягт
labChipCalendarLlm	日历 LLM	日曆 LLM	日曆 LLM	カレンダー LLM	달력 LLM	ปฏิทิน LLM	lịch LLM	kalender LLM	kalendar LLM	хуанли LLM
labChipQuietAck	短确认	短確認	短確認	短い確認	짧은 확인	ยืนยันสั้น	xác nhận ngắn	konfirmasi singkat	pengesahan ringkas	богино баталгаа
labChipLlmTools	LLM 工具	LLM 工具	LLM 工具	LLM ツール	LLM 도구	เครื่องมือ LLM	công cụ LLM	alat LLM	alat LLM	LLM хэрэгсэл
labChipLlmChat	LLM 对话	LLM 對話	LLM 傾偈	LLM チャット	LLM 채팅	แชท LLM	trò chuyện LLM	obrolan LLM	sembang LLM	LLM чат
labChipConfirmRisky	确认风险	確認風險	確認風險	リスク確認	위험 확인	ยืนยันความเสี่ยง	xác nhận rủi ro	konfirmasi risiko	sahkan risiko	эрсдэл батлах
labDecisionExecute	Klar 执行	Klar 執行	Klar 執行	Klar 実行	Klar 실행	Klar รัน	Klar thực hiện	Klar jalankan	Klar laksana	Klar гүйцэтгэх
labDecisionBriefing	简报	簡報	簡報	ブリーフィング	브리핑	สรุป	tóm tắt	pengarahan	taklimat	товч
labParse	Klar 解析	Klar 解析	Klar 解析	Klar 解析	Klar 분석	แยก Klar	Phân Klar	Uraian Klar	Huraian Klar	Klar задлал
reasonMissingArea	缺房间	缺房間	缺房	部屋なし	방 없음	ไม่มีห้อง	thiếu phòng	tanpa ruang	tiada bilik	өрөөгүй
reasonWeakName	弱名称	弱名稱	弱名	弱い名前	약한 이름	ชื่ออ่อน	tên yếu	nama lemah	nama lemah	сул нэр
reasonReady	就绪	就緒	就位	準備完了	준비됨	พร้อม	sẵn	siap	sedia	бэлэн
reasonMatch	匹配	相符	啱	一致	일치	ตรง	khớp	cocok	padan	тохирол
sentencesEmpty	一句落到已知意图。不是每盏灯一句。	一句落到已知意圖。不是每盞燈一句。	一句落到已知意圖。唔係每盞燈一句。	既知の意図へ一文。照明ごとに一句ではない。	알려진 의도에 한 문장. 조명마다 한 구가 아닙니다.	หนึ่งประโยคสู่เจตนาที่รู้จัก ไม่ใช่หนึ่งวลีต่อดวงไฟ	Một câu vào ý định đã biết. Không một cụm mỗi đèn.	Satu kalimat ke niat yang dikenal. Bukan satu frasa per lampu.	Satu ayat ke niat dikenali. Bukan satu frasa setiap lampu.	Мэдэгдэх зорилгод нэг өгүүлбэр. Гэрэл бүрт нэг хэллэг биш.
policiesEmpty	第一条匹配的规则获胜。选设备、房间或楼层 — 不要输入 id。	第一條符合的規則獲勝。選裝置、房間或樓層 — 不要輸入 id。	第一條啱嘅規則贏。揀裝置、房或者樓層 — 唔好打 id。	最初に一致した規則が勝つ。機器・部屋・階を選ぶ。id は打たない。	첫 일치 규칙이 이깁니다. 기기, 방 또는 층을 고르세요. id를 입력하지 마세요.	กฎที่ตรงข้อแรกชนะ เลือกอุปกรณ์ ห้อง หรือชั้น — อย่าพิมพ์ id	Quy tắc khớp đầu tiên thắng. Chọn thiết bị, phòng hoặc tầng — đừng gõ id.	Aturan cocok pertama menang. Pilih perangkat, ruang, atau lantai — jangan ketik id.	Peraturan sepadan pertama menang. Pilih peranti, bilik atau tingkat — jangan taip id.	Эхний таарсан дүрэм ялна. Төхөөрөмж, өрөө, давхар сонго — id битгий бич.
setupAgainWhere	重放设置在「设置」里。	重播設定在「設定」裡。	重播設定喺「設定」。	セットアップの再実行は設定にあります。	설정 다시는 설정에 있습니다.	ตั้งค่าอีกครั้งอยู่ในการตั้งค่า	Phát lại setup nằm trong Cài đặt.	Ulang setup ada di Pengaturan.	Ulang setup ada dalam Tetapan.	Суулгалтыг дахин тоглуулах нь Тохиргоонд бий.
whyThisBand	为何是这个频段？	為何是這個頻段？	點解呢個頻段？	なぜこの帯域？	왜 이 대역인가요?	ทำไมแถบนี้	Vì sao dải này?	Mengapa pita ini?	Mengapa jalur ini?	Яагаад энэ зурвас вэ?
rememberAsPhrase	记为短语	記為語句	記做語句	フレーズとして覚える	구로 기억	จำเป็นวลี	Nhớ thành cụm	Ingat sebagai frasa	Ingat sebagai frasa	Хэллэгээр сана
evidence	证据	證據	證據	根拠	근거	หลักฐาน	bằng chứng	bukti	bukti	нотолбар
names	名称	名稱	名	名前	이름	ชื่อ	tên	nama	nama	нэр
settingsBackup	设置备份	設定備份	設定備份	設定のバックアップ	설정 백업	สำรองการตั้งค่า	Sao lưu cài đặt	Cadangan pengaturan	Sandaran tetapan	Тохиргооны нөөц
settingsBackupHint	覆盖层、分配、规则和 LLM 端点。对话和房屋图留在本引擎。	覆蓋層、指派、規則和 LLM 端點。對話和住家圖留在本引擎。	覆蓋層、指派、規則同 LLM 端點。對話同屋企圖留喺呢個引擎。	オーバーレイ、割り当て、規則、LLM エンドポイント。会話と家グラフはこのエンジンに残ります。	오버레이, 할당, 규칙, LLM 엔드포인트. 대화와 집 그래프는 이 엔진에 남습니다.	โอเวอร์เลย์ การกำหนด กฎ และจุดปลาย LLM บทสนทนาและกราฟบ้านอยู่บนเอนจินนี้	Lớp phủ, gán, quy tắc và điểm cuối LLM. Hội thoại và đồ thị nhà ở lại trên động cơ này.	Hamparan, penetapan, aturan, dan titik akhir LLM. Percakapan dan graf rumah tetap di mesin ini.	Hamparan, penugasan, peraturan dan titik akhir LLM. Perbualan dan graf rumah kekal pada enjin ini.	Давхарга, оноолт, дүрэм, LLM төгсгөл. Яриа болон гэрийн граф энэ хөдөлгүүрт үлдэнэ.
settingsBackupDownload	下载设置	下載設定	下載設定	設定をダウンロード	설정 내려받기	ดาวน์โหลดการตั้งค่า	Tải cài đặt	Unduh pengaturan	Muat turun tetapan	Тохиргоо татах
settingsBackupIncludeKey	包含 API 密钥	包含 API 金鑰	包含 API 鎖匙	API キーを含める	API 키 포함	รวมคีย์ API	Gồm khóa API	Sertakan kunci API	Sertakan kunci API	API түлхүүр оруулах
settingsBackupIncludeKeyHint	默认关闭。开启：归档含已存的 LLM 密钥。	預設關閉。開啟：封存含已存的 LLM 金鑰。	預設關。開：封存有存低嘅 LLM 鎖匙。	既定はオフ。オン：アーカイブに保存済み LLM キーが入る。	기본 꺼짐. 켜면: 보관에 저장된 LLM 키가 들어갑니다.	ปิดเป็นค่าเริ่ม เปิด: คลังมีคีย์ LLM ที่เก็บไว้	Tắt mặc định. Bật: kho chứa khóa LLM đã lưu.	Nonaktif bawaan. Aktif: arsip berisi kunci LLM tersimpan.	Mati lalai. Hidup: arkib mengandungi kunci LLM tersimpan.	Анхдагч унтраалттай. Асаалттай: архив хадгалсан LLM түлхүүртэй.
settingsBackupIncludeKeyConfirm	归档将包含 LLM 的 API 密钥。仍要下载？	封存將包含 LLM 的 API 金鑰。仍要下載？	封存會有 LLM 嘅 API 鎖匙。都下載？	アーカイブに LLM の API キーが入ります。それでもダウンロードしますか？	보관에 LLM API 키가 들어갑니다. 그래도 받을까요?	คลังจะมีคีย์ API ของ LLM ดาวน์โหลดอยู่ดีไหม	Kho sẽ chứa khóa API của LLM. Vẫn tải?	Arsip akan berisi kunci API LLM. Unduh tetap?	Arkib akan mengandungi kunci API LLM. Muat turun juga?	Архив LLM-ийн API түлхүүртэй болно. Гэсэн ч татах уу?
settingsBackupRestore	恢复设置	還原設定	還原設定	設定を復元	설정 복원	คืนค่าการตั้งค่า	Khôi phục cài đặt	Pulihkan pengaturan	Pulihkan tetapan	Тохиргоо сэргээх
settingsBackupRestoreConfirm	这会覆盖此引擎上的全部 Klar 设置。	這會覆蓋此引擎上的全部 Klar 設定。	呢個會覆蓋呢個引擎上面全部 Klar 設定。	このエンジン上のすべての Klar 設定を上書きします。	이 엔진의 모든 Klar 설정을 덮어씁니다.	นี่จะเขียนทับการตั้งค่า Klar ทั้งหมดบนเอนจินนี้	Việc này ghi đè mọi cài đặt Klar trên động cơ này.	Ini menimpa semua pengaturan Klar di mesin ini.	Ini menimpa semua tetapan Klar pada enjin ini.	Энэ нь энэ хөдөлгүүр дээрх бүх Klar тохиргоог дарж бичнэ.
settingsBackupRestoreOk	设置已恢复。	設定已還原。	設定已還原。	設定を復元しました。	설정을 복원했습니다.	คืนค่าการตั้งค่าแล้ว	Đã khôi phục cài đặt.	Pengaturan dipulihkan.	Tetapan dipulihkan.	Тохиргоо сэргээгдлээ.
settingsBackupRestoreFail	无法恢复设置。	無法還原設定。	還原唔到設定。	設定を復元できませんでした。	설정을 복원하지 못했습니다.	คืนค่าการตั้งค่าไม่สำเร็จ	Không khôi phục được cài đặt.	Gagal memulihkan pengaturan.	Tidak dapat pulihkan tetapan.	Тохиргоо сэргээж чадсангүй.
settingsBackupPickFile	选择归档	選擇封存	揀封存	アーカイブを選ぶ	보관 선택	เลือกคลัง	Chọn kho	Pilih arsip	Pilih arkib	Архив сонгох
speechRefined	LLM 润色	LLM 潤飾	LLM 潤飾	LLM 洗練	LLM 다듬기	LLM ปรับ	tinh LLM	perhalus LLM	perhalusi LLM	LLM нягт
speechChat	LLM 对话	LLM 對話	LLM 傾偈	LLM チャット	LLM 채팅	แชท LLM	trò chuyện LLM	obrolan LLM	sembang LLM	LLM чат
refineRejected	润色保留了 NLU 回复。	潤飾保留了 NLU 回覆。	潤飾留低咗 NLU 回覆。	洗練は NLU の返答を残しました。	다듬기가 NLU 응답을 유지했습니다.	การปรับคงคำตอบ NLU	Tinh giữ câu trả lời NLU.	Perhalus mempertahankan balasan NLU.	Perhalusi mengekalkan balasan NLU.	Нягт нь NLU хариултыг үлдээсэн.
"""

PACKS = parse_rows(CODES, TABLE)
