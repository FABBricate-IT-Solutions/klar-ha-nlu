"""Operator UI chrome for East and Southeast Asian Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_table

CODES = ["zh-CN", "zh-TW", "zh-HK", "ja", "ko", "th", "vi", "id", "ms", "mn"]

TABLE = """
home	首页	首頁	主頁	ホーム	홈	หน้าแรก	Trang chủ	Beranda	Laman	Нүүр
conversations	对话	對話	對話	会話	대화	สนทนา	Hội thoại	Percakapan	Perbualan	Яриа
rules	规则	規則	規則	ルール	규칙	กฎ	Quy tắc	Aturan	Peraturan	Дүрэм
house	住宅	住家	屋企	家	집	บ้าน	Nhà	Rumah	Rumah	Гэр
lab	实验室	實驗室	實驗室	ラボ	랩	ทดลอง	Thí nghiệm	Uji	Makmal	Туршилт
graph	图谱	圖譜	圖譜	グラフ	그래프	กราฟ	Đồ thị	Graf	Graf	Граф
calibrate	对照	對應	對應	対応	대응	การจับคู่	Ánh xạ	Pemetaan	Pemetaan	Буулгалт
entities	设备	裝置	裝置	機器	기기	อุปกรณ์	Thiết bị	Perangkat	Peranti	Төхөөрөмж
custom	短语	語句	語句	フレーズ	구문	วลี	Cụm từ	Frasa	Frasa	Хэллэг
settings	设置	設定	設定	設定	설정	การตั้งค่า	Cài đặt	Pengaturan	Tetapan	Тохиргоо
open	待处理	待處理	待處理	未処理	미처리	ค้าง	mở	terbuka	terbuka	нээлттэй
bundleOn	捆绑开启	套件開啟	套件開啟	バンドルオン	번들 켜짐	ชุดเปิดอยู่	Gói đang bật	Bundel nyala	Bundel hidup	Багц асаалттай
bundleOff	捆绑关闭	套件關閉	套件關閉	バンドルオフ	번들 꺼짐	ชุดปิดอยู่	Gói đang tắt	Bundel mati	Bundel mati	Багц унтраалттай
engineReady	引擎就绪	引擎就緒	引擎就緒	エンジン準備完了	엔진 준비됨	เอนจินพร้อม	Động cơ sẵn	Mesin siap	Enjin sedia	Хөдөлгүүр бэлэн
understandsHome	Klar 为何执行、确认或停止	Klar 為何執行、確認或停止	Klar 點解執行、確認或停	Klar が実行・確認・停止した理由	Klar가 실행·확인·중단한 이유	เหตุที่ Klar ทำ ยืนยัน หรือหยุด	Vì sao Klar chạy, xác nhận hoặc dừng	Mengapa Klar menjalankan, mengonfirmasi, atau berhenti	Mengapa Klar laksana, sahkan, atau henti	Klar яагаад ажиллуулсан, баталсан, зогсоосон
assistVisible	Assist 可见	Assist 可見	Assist 睇到	Assist 表示	Assist 표시	Assist มองเห็น	Assist hiện	Assist terlihat	Assist kelihatan	Assist харагдана
certain	确定	確定	確定	確実	확실	แน่นอน	chắc	yakin	yakin	итгэлтэй
needsWork	需调整	需調整	要執	要調整	손봐야 함	ต้องปรับ	cần sửa	perlu kerja	perlu kerja	засах хэрэгтэй
recordings	记录	紀錄	紀錄	記録	기록	บันทึก	Bản ghi	Rekaman	Rakaman	Бичлэг
processed	已处理	已處理	處理咗	処理済み	처리됨	ประมวลแล้ว	đã xử lý	diproses	diproses	боловсруулсан
coverage	覆盖率	涵蓋率	涵蓋率	カバー率	적용률	ความครอบคลุม	Phủ	Cakupan	Liputan	Хамрах
confidence	置信度	信心度	信心度	確信度	신뢰도	ความมั่นใจ	Độ tin	Keyakinan	Keyakinan	Итгэл
domains	领域	領域	領域	ドメイン	도메인	โดเมน	Lĩnh vực	Ranah	Bidang	Домэйн
rooms	房间	房間	房間	部屋	방	ห้อง	Phòng	Ruangan	Bilik	Өрөө
recent	最近句子	最近句子	最近句子	最近の文	최근 문장	ประโยคล่าสุด	Câu gần đây	Kalimat terbaru	Ayat terbaru	Сүүлийн өгүүлбэр
replay	重放	重播	重播	再生	다시 듣기	เล่นซ้ำ	Phát lại	Putar ulang	Main semula	Дахин
applyAll	应用建议	套用建議	套用建議	提案を適用	제안 적용	ใช้ข้อเสนอ	Áp gợi ý	Terapkan saran	Guna cadangan	Санал хэрэглэх
undo	撤销	復原	復原	元に戻す	실행 취소	เลิกทำ	Hoàn tác	Urungkan	Buat asal	Буцаах
accept	接受	接受	接受	採用	수락	ยอมรับ	Chấp nhận	Terima	Terima	Зөвшөөрөх
otherRoom	其他房间	其他房間	第個房	別の部屋	다른 방	ห้องอื่น	Phòng khác	Ruang lain	Bilik lain	Өөр өрөө
dismiss	忽略	略過	略過	閉じる	닫기	ปิด	Bỏ	Abaikan	Abai	Хаах
noGaps	没有未对照。	沒有未對應。	冇未對應。	未対応はありません。	열린 대응 없음.	ไม่มีการจับคู่ว่าง	Không còn ánh xạ mở.	Tidak ada pemetaan terbuka.	Tiada pemetaan terbuka.	Нээлттэй буулгалт алга.
unmapped	无房间	無房間	冇房	部屋なし	방 없음	ไม่มีห้อง	Không phòng	Tanpa ruang	Tiada bilik	Өрөөгүй
parseHint	句子触发器在此次解析前于 Home Assistant 中运行。conversation.process：先触发，再 Klar，然后 intent_script。	句子觸發器在此次解析前於 Home Assistant 中執行。conversation.process：先觸發，再 Klar，然後 intent_script。	句子觸發器喺今次解析前喺 Home Assistant 跑。conversation.process：先觸發，再 Klar，然後 intent_script。	文トリガーはこの解析の前に Home Assistant で実行されます。conversation.process：トリガー、次に Klar、その後 intent_script。	문장 트리거는 이 분석 전에 Home Assistant에서 실행됩니다. conversation.process: 트리거, 그다음 Klar, 그다음 intent_script.	ทริกเกอร์ประโยคทำงานใน Home Assistant ก่อนการแยกนี้ conversation.process: ทริกเกอร์ แล้ว Klar แล้ว intent_script	Bộ kích câu chạy trong Home Assistant trước lần phân này. conversation.process: kích, rồi Klar, rồi intent_script.	Pemicu kalimat berjalan di Home Assistant sebelum uraian ini. conversation.process: pemicu, lalu Klar, lalu intent_script.	Pencetus ayat jalan dalam Home Assistant sebelum huraian ini. conversation.process: pencetus, lalu Klar, lalu intent_script.	Өгүүлбэрийн триггер энэ задлалаас өмнө Home Assistant-д ажиллана. conversation.process: триггер, дараа Klar, дараа intent_script.
command	命令	指令	指令	コマンド	명령	คำสั่ง	Lệnh	Perintah	Arahan	Тушаал
analyze	分析	分析	分析	解析	분석	วิเคราะห์	Phân tích	Analisis	Analisis	Шинжлэх
raw	原始	原始	原始	生	원문	ดิบ	Thô	Mentah	Mentah	Түүхий
speech	语音	語音	語音	発話	음성	คำพูด	Lời nói	Ucapan	Pertuturan	Яриа
intent	意图	意圖	意圖	意図	의도	เจตนา	Ý định	Niat	Niat	Зорилго
slots	槽位	槽位	槽位	スロット	슬롯	ช่อง	Ô	Bidang	Medan	Нүд
searchDevice	你会怎么叫它？	你會怎麼叫它？	你會點叫佢？	何と呼びますか？	뭐라고 부르시겠어요?	จะเรียกว่าอะไร	Bạn gọi nó là gì?	Anda menyebutnya apa?	Anda panggil apa?	Юу гэж нэрлэх вэ?
alias	别名	別名	別名	別名	별칭	ชื่อเรียก	Bí danh	Nama lain	Nama lain	Өөр нэр
room	房间	房間	房間	部屋	방	ห้อง	Phòng	Ruangan	Bilik	Өрөө
preferred	默认灯	預設燈	預設燈	既定の照明	기본 조명	ไฟหลัก	Đèn mặc định	Lampu bawaan	Lampu lalai	Үндсэн гэрэл
save	保存	儲存	儲存	保存	저장	บันทึก	Lưu	Simpan	Simpan	Хадгалах
personality	人格	人格	人格	人格	성격	บุคลิก	Tính cách	Kepribadian	Personaliti	Зан чанар
mode	模式	模式	模式	モード	모드	โหมด	Chế độ	Mode	Mod	Горим
supportBundle	支持捆绑	支援套件	支援套件	サポートバンドル	지원 번들	ชุดสนับสนุน	Gói hỗ trợ	Bundel dukungan	Bundel sokongan	Дэмжлэгийн багц
recordProtocol	记录协议	紀錄協定	紀錄協定	プロトコル記録	프로토콜 기록	บันทึกโปรโตคอล	Ghi giao thức	Rekam protokol	Rakam protokol	Протокол бичих
includeRawText	下载中包含原文	下載時包含原文	下載時包含原文	ダウンロードに原文を含める	다운로드에 원문 포함	รวมข้อความดิบในไฟล์ดาวน์โหลด	Gồm văn thô khi tải	Sertakan teks mentah di unduhan	Sertakan teks mentah dalam muat turun	Татахад түүхий текст оруулах
semanticAdapters	本地语义适配器	本機語意適配器	本機語意適配器	ローカル意味アダプター	로컬 의미 어댑터	อะแดปเตอร์ความหมายในเครื่อง	Bộ chuyển ngữ nghĩa cục bộ	Adaptor semantik lokal	Penyesuai semantik setempat	Орон нутгийн утгын адаптер
downloadDataset	下载数据集	下載資料集	下載資料集	データセットをダウンロード	데이터셋 다운로드	ดาวน์โหลดชุดข้อมูล	Tải bộ dữ liệu	Unduh kumpulan data	Muat turun set data	Өгөгдлийн багц татах
downloadProtocol	下载协议	下載協定	下載協定	プロトコルをダウンロード	프로토콜 다운로드	ดาวน์โหลดโปรโตคอล	Tải giao thức	Unduh protokol	Muat turun protokol	Протокол татах
deleteSelected	删除所选	刪除所選	刪除所選	選択を削除	선택 삭제	ลบที่เลือก	Xóa mục chọn	Hapus pilihan	Padam pilihan	Сонгосныг устгах
clearAll	全部删除	全部刪除	全部刪除	すべて削除	모두 삭제	ลบทั้งหมด	Xóa hết	Hapus semua	Padam semua	Бүгдийг устгах
token	写入令牌（LAN）	寫入權杖（LAN）	寫入權杖（LAN）	書き込みトークン（LAN）	쓰기 토큰(LAN)	โทเคนเขียน (LAN)	Token ghi (LAN)	Token tulis (LAN)	Token tulis (LAN)	Бичих токен (LAN)
customJson	自定义短语（JSON）	自訂語句（JSON）	自訂語句（JSON）	カスタムフレーズ（JSON）	사용자 구문(JSON)	วลีกำหนดเองเป็น JSON	Cụm từ tùy chỉnh dạng JSON	Frasa khusus sebagai JSON	Frasa tersuai sebagai JSON	Өөрийн хэллэг JSON-аар
customHint	把短语接到已知意图。策略在旁边，不是 HA 自动化。	把語句接到已知意圖。原則在旁邊，不是 HA 自動化。	將語句接到已知意圖。策略喺隔離，唔係 HA 自動化。	既知の意図にフレーズを結びます。方針はこの横であり、HA オートメーションではありません。	알려진 의도에 구문을 연결합니다. 정책은 옆에 있으며 HA 자동화로 두지 않습니다.	ผูกวลีกับเจตนาที่รู้จัก นโยบายอยู่ข้างนี้ ไม่ใช่ระบบอัตโนมัติของ HA	Gắn cụm từ vào ý định đã biết. Chính sách nằm bên cạnh, không phải tự động hóa HA.	Pasangkan frasa ke niat yang dikenal. Kebijakan di samping ini, bukan otomasi HA.	Pasangkan frasa ke niat yang diketahui. Dasar di sebelah ini, bukan automasi HA.	Мэдэгдэх зорилгод хэллэг холбоно. Бодлого хажууд байна, HA автомат биш.
addPhrase	添加短语	新增語句	新增語句	フレーズを追加	구문 추가	เพิ่มวลี	Thêm cụm từ	Tambah frasa	Tambah frasa	Хэллэг нэмэх
previewRule	预览	預覽	預覽	プレビュー	미리보기	ดูตัวอย่าง	Xem trước	Pratinjau	Pratonton	Урьдчилан
explainRule	解释	說明	解釋	説明	설명	อธิบาย	Giải thích	Jelaskan	Terangkan	Тайлбарлах
rollback	回滚	回溯	回溯	ロールバック	되돌리기	ย้อนกลับ	Hoàn lui	Gulung balik	Gulung balik	Буцаах
noRules	还没有自定义短语。	還沒有自訂語句。	未有自訂語句。	カスタムフレーズはまだありません。	사용자 구문이 아직 없습니다.	ยังไม่มีวลีกำหนดเอง	Chưa có cụm từ tùy chỉnh.	Belum ada frasa khusus.	Belum ada frasa tersuai.	Өөрийн хэллэг хараахан алга.
engineOffline	引擎无法连接。在成功实时加载之前，此列表为空。	引擎無法連線。在成功即時載入之前，此列表為空。	引擎無法連線。成功即時載入之前，此列表為空。	エンジンに到達できません。ライブ読み込みが成功するまで、この一覧は空です。	엔진에 연결할 수 없습니다. 실시간 불러오기가 성공할 때까지 이 목록은 비어 있습니다.	เชื่อมต่อเอนจินไม่ได้ รายการนี้ว่างจนกว่าการโหลดสดจะสำเร็จ	Không thể kết nối máy. Danh sách trống cho đến khi tải trực tiếp thành công.	Mesin tidak terjangkau. Daftar ini kosong sampai pemuatan langsung berhasil.	Enjin tidak dapat dicapai. Senarai ini kosong sehingga muatan langsung berjaya.	Хөдөлгүүрт холбогдохгүй. Шууд ачаалал амжилттай болтол жагсаалт хоосон.
emptyBundle	还没有记录。请开启捆绑并试一句。	還沒有紀錄。請開啟套件並試一句。	未有紀錄。請開套件試一句。	記録はまだありません。バンドルを有効にして文を試してください。	기록이 아직 없습니다. 번들을 켜고 문장을 시험하세요.	ยังไม่มีบันทึก เปิดชุดแล้วลองประโยค	Chưa có bản ghi. Bật gói và thử một câu.	Belum ada rekaman. Nyalakan bundel dan coba sebuah kalimat.	Belum ada rakaman. Hidupkan bundel dan cuba satu ayat.	Бичлэг хараахан алга. Багцыг асаагаад өгүүлбэр туршина уу.
confirmApply	应用这些建议？	套用這些建議？	套用呢啲建議？	これらの提案を適用しますか？	이 제안을 적용할까요?	ใช้ข้อเสนอเหล่านี้หรือไม่	Áp các gợi ý này?	Terapkan saran ini?	Guna cadangan ini?	Эдгээр саналыг хэрэглэх үү?
cancel	取消	取消	取消	キャンセル	취소	ยกเลิก	Hủy	Batal	Batal	Цуцлах
apply	应用	套用	套用	適用	적용	ใช้	Áp	Terapkan	Guna	Хэрэглэх
close	关闭	關閉	關閉	閉じる	닫기	ปิด	Đóng	Tutup	Tutup	Хаах
low	低	低	低	低	낮음	ต่ำ	thấp	rendah	rendah	бага
medium	中	中	中	中	중간	กลาง	vừa	sedang	sederhana	дунд
high	高	高	高	高	높음	สูง	cao	tinggi	tinggi	өндөр
source	来源	來源	來源	出典	출처	แหล่ง	Nguồn	Sumber	Sumber	Эх
language	语言	語言	語言	言語	언어	ภาษา	Ngôn ngữ	Bahasa	Bahasa	Хэл
time	时间	時間	時間	時刻	시각	เวลา	Thời gian	Waktu	Masa	Цаг
text	句子	句子	句子	文	문장	ประโยค	Câu	Kalimat	Ayat	Өгүүлбэр
answer	回答	回答	回答	返答	답변	คำตอบ	Trả lời	Jawaban	Jawapan	Хариу
graphHint	房间为簇，设备按置信度着色。	房間為簇，裝置依信心度上色。	房間係簇，裝置按信心度上色。	部屋はクラスタ、機器は確信度で色分け。	방은 군집, 기기는 신뢰도로 색칠.	ห้องเป็นกลุ่ม อุปกรณ์ย้อมตามความมั่นใจ	Phòng thành cụm, thiết bị tô theo độ tin.	Ruangan sebagai kelompok, perangkat diwarnai menurut keyakinan.	Bilik sebagai kelompok, peranti diwarnai ikut keyakinan.	Өрөөнүүд бөөгнөрөл, төхөөрөмжийг итгэлээр будна.
resetLayout	重置布局	重設版面	重設版面	配置をリセット	배치 초기화	จัดวางใหม่	Đặt lại bố cục	Atur ulang tata letak	Set semula susun atur	Байрлал шинэчлэх
score	分数	分數	分數	スコア	점수	คะแนน	Điểm	Nilai	Nilai	Оноо
noIntent	无意图	無意圖	冇意圖	意図なし	의도 없음	ไม่มีเจตนา	Không ý định	Tidak ada niat	Tiada niat	Зорилгогүй
loading	正在加载 Klar...	正在載入 Klar...	而家載入 Klar...	Klar を読み込み中...	Klar 불러오는 중...	กำลังโหลด Klar...	Đang tải Klar...	Memuat Klar...	Memuatkan Klar...	Klar ачаалж байна...
nluRagHint	默认关闭。仅已匹配片段，绝不用 Assist 工具。	預設關閉。僅已比對片段，絕不用 Assist 工具。	預設關閉。只係已配對片段，絕唔用 Assist 工具。	既定はオフ。一致した断片のみ、Assist ツールは使いません。	기본은 꺼짐. 맞춘 조각만, Assist 도구는 쓰지 않습니다.	ปิดเป็นค่าเริ่มต้น ส่งเฉพาะช่วงที่จับได้ ไม่ใช้เครื่องมือ Assist	Tắt mặc định. Chỉ lát đã khớp, không bao giờ công cụ Assist.	Mati secara bawaan. Hanya potongan yang cocok, tidak pernah alat Assist.	Mati secara lalai. Hanya potongan yang sepadan, tidak pernah alat Assist.	Анхдагч унтраалттай. Зөвхөн таарсан хэсэг, Assist хэрэгсэл хэзээ ч үгүй.
confirmRisky	确认危险操作	確認危險操作	確認危險操作	危険な操作を確認	위험한 동작 확인	ยืนยันการกระทำเสี่ยง	Xác nhận thao tác rủi ro	Konfirmasi aksi berisiko	Sahkan tindakan berisiko	Эрсдэлтэй үйлдлийг батлах
languages	语言	語言	語言	言語	언어	ภาษา	Ngôn ngữ	Bahasa	Bahasa	Хэлнүүд
languageSearch	搜索语言	搜尋語言	搜尋語言	言語を検索	언어 검색	ค้นหาภาษา	Tìm ngôn ngữ	Cari bahasa	Cari bahasa	Хэл хайх
allLanguages	全部语言	所有語言	所有語言	すべての言語	모든 언어	ทุกภาษา	Mọi ngôn ngữ	Semua bahasa	Semua bahasa	Бүх хэл
noLanguageMatch	未找到语言	找不到語言	搵唔到語言	言語が見つかりません	언어를 찾지 못함	ไม่พบภาษา	Không thấy ngôn ngữ	Bahasa tidak ditemukan	Bahasa tidak dijumpai	Хэл олдсонгүй
languageHint	搜索并选择语言区域。全部语言会启用每个已编译语言包。	搜尋並選擇語系。所有語言會啟用每個已編譯語言包。	搜尋並揀語系。所有語言會開晒每個已編譯語言包。	検索してロケールを選びます。すべての言語ではコンパイル済みパックをすべて有効にします。	검색해 로캘을 고르세요. 모든 언어는 컴파일된 팩을 모두 켭니다.	ค้นแล้วเลือกภาษาท้องถิ่น ทุกภาษาจะเปิดทุกแพ็กที่คอมไพล์แล้ว	Tìm và chọn vùng ngôn ngữ. Mọi ngôn ngữ giữ mọi gói đã biên dịch.	Cari dan pilih lokal. Semua bahasa menjaga setiap paket terkompilasi tetap aktif.	Cari dan pilih lokal. Semua bahasa mengekalkan setiap pek terkompil aktif.	Хайж локал сонгоно. Бүх хэл нь эмхэтгэсэн багц бүрийг идэвхтэй байлгана.
mappingHint	对照是图谱实体的别名。纳入日历领域后才会出现日历。Assist 跟随语言包；此界面跟随操作员语言。	對應是圖譜實體的別名。納入日曆領域後才會出現日曆。Assist 跟隨語言包；此介面跟隨操作者語言。	對應係圖譜實體嘅別名。納入日曆領域之後先有日曆。Assist 跟語言包；呢個介面跟操作員語言。	対応はグラフ実体の別名です。カレンダー領域を含めた後にカレンダーが表示されます。Assist は言語パックに従い、この画面は操作者の言語に従います。	대응은 그래프 엔티티의 별칭입니다. 캘린더 도메인을 포함한 뒤에 캘린더가 나타납니다. Assist는 언어 팩을 따르고, 이 화면은 운영자 언어를 따릅니다.	การจับคู่คือชื่อเรียกของเอนทิตีในกราฟ ปฏิทินจะปรากฏเมื่อรวมโดเมนปฏิทินแล้ว Assist ตามชุดภาษา หน้านี้ตามภาษาของผู้ดูแล	Ánh xạ là bí danh cho thực thể đồ thị. Lịch hiện sau khi gồm lĩnh vực lịch. Assist theo gói ngôn ngữ; giao diện này theo ngôn ngữ người vận hành.	Pemetaan adalah nama lain untuk entitas graf. Kalender muncul setelah ranah kalender disertakan. Assist mengikuti paket bahasa; tampilan ini mengikuti bahasa pengelola.	Pemetaan ialah nama lain untuk entiti graf. Kalendar muncul selepas bidang kalendar disertakan. Assist mengikut pek bahasa; paparan ini mengikut bahasa pengendali.	Буулгалт нь график нэгжийн өөр нэр. Хуанлийн домэйныг оруулсны дараа хуанли гарна. Assist хэлний багцыг дагана; энэ дэлгэц операторын хэлийг дагана.
parseSample	打开客厅灯	打開客廳燈	開客廳燈	リビングの電気をつけて	거실 불 켜줘	เปิดไฟห้องนั่งเล่น	Bật đèn phòng khách	Nyalakan lampu ruang tamu	Hidupkan lampu ruang tamu	Зочны өрөөний гэрлийг асаа
tryOn	打开{room}的灯	打開{room}的燈	開{room}嘅燈	{room}の電気をつけて	{room} 불 켜줘	เปิดไฟที่ {room}	Bật đèn ở {room}	Nyalakan lampu di {room}	Hidupkan lampu di {room}	{room} дахь гэрлийг асаа
tryLock	门锁上了吗？	門鎖上了嗎？	門鎖咗未？	ドアは鍵がかかっていますか？	문 잠겼어요?	ประตูล็อกหรือยัง	Cửa có khóa không?	Pintunya terkunci?	Pintu berkunci?	Хаалга түгжигдсэн үү?
tryTime	现在几点？	現在幾點？	而家幾點？	今何時ですか？	지금 몇 시예요?	กี่โมงแล้ว	Mấy giờ rồi?	Jam berapa sekarang?	Pukul berapa sekarang?	Хэдэн цаг болж байна?
tryNight	晚安	晚安	晚安	おやすみ	잘 자	ราตรีสวัสดิ์	Chúc ngủ ngon	Selamat malam	Selamat malam	Сайхан амраарай
tryUndo	撤销刚才的	復原剛才的	復原頭先嗰句	それを取り消して	그거 취소해	เลิกทำอันนั้น	Hoàn tác cái đó	Urungkan itu	Buat asal itu	Тэрийг буцаа
tryRoom	厨房	廚房	廚房	キッチン	부엌	ห้องครัว	nhà bếp	dapur	dapur	галтогоо
nluIgnore	不为状态或电源绑定	不為狀態或電源綁定	唔為狀態或電源綁定	状態や電源には結びつけない	상태나 전원에 묶지 않음	ไม่ผูกสำหรับสถานะหรือไฟ	Không gắn cho trạng thái hoặc nguồn	Jangan ikat untuk status atau daya	Jangan ikat untuk status atau kuasa	Төлөв эсвэл тэжээлд бүү холбо
nluIgnoreHint	从解析器中排除此设备。用于名称错误的辅助项。	從解析器排除此裝置。用於名稱錯誤的輔助項。	從解析器剔除呢部裝置。用喺改錯名嘅輔助項。	解決器からこの機器を外します。名前が誤ったヘルパーに使います。	해석기에서 이 기기를 뺍니다. 이름 잘못된 도우미에 쓰세요.	ตัดอุปกรณ์นี้ออกจากตัวจับคู่ ใช้กับตัวช่วยที่ตั้งชื่อผิด	Gỡ thiết bị này khỏi bộ giải. Dùng cho trợ giúp đặt sai tên.	Keluarkan perangkat ini dari penyelesai. Untuk pembantu yang salah nama.	Keluarkan peranti ini daripada penyelesai. Untuk pembantu yang salah nama.	Энэ төхөөрөмжийг шийдэгчээс хасна. Буруу нэртэй туслагчид хэрэглэнэ.
savePhrase	存为短语	存成語句	存成語句	フレーズとして保存	구문으로 저장	บันทึกเป็นวลี	Lưu thành cụm từ	Simpan sebagai frasa	Simpan sebagai frasa	Хэллэгээр хадгалах
ignoreTarget	忽略此目标	略過此目標	略過呢個目標	この対象を無視	이 대상 무시	ละเว้นเป้าหมายนี้	Bỏ mục tiêu này	Abaikan sasaran ini	Abai sasaran ini	Энэ зорилтыг үл тоомсорлох
teachSaved	已保存。	已儲存。	儲存咗。	保存しました。	저장했습니다.	บันทึกแล้ว	Đã lưu.	Tersimpan.	Disimpan.	Хадгаллаа.
journal	对话日志	對話日誌	對話日誌	会話日誌	대화 일지	บันทึกสนทนา	Nhật ký hội thoại	Jurnal percakapan	Jurnal perbualan	Ярианы тэмдэглэл
journalHint	最近 200 轮、24 小时、已脱敏。原文仅随捆绑提供。	最近 200 輪、24 小時、已遮罩。原文只隨套件提供。	最近 200 輪、24 小時、已遮罩。原文只跟套件。	直近 200 往復、24 時間、伏せ字。原文はバンドル時のみ。	최근 200회, 24시간, 가림. 원문은 번들과만.	200 รอบล่าสุด 24 ชั่วโมง ปิดบัง ข้อความดิบเฉพาะตอนเปิดชุด	200 lượt gần nhất, 24 giờ, đã che. Văn thô chỉ kèm gói.	200 giliran terakhir, 24 jam, disamarkan. Teks mentah hanya bersama bundel.	200 giliran terakhir, 24 jam, disunting. Teks mentah hanya bersama bundel.	Сүүлийн 200 ээлж, 24 цаг, нууцласан. Түүхий текст зөвхөн багцтай.
decisionMix	判定	判定	判定	判定	결정	การตัดสิน	Quyết định	Keputusan	Keputusan	Шийдвэр
mixCaption	来源：对话日志，每日轮次	來源：對話日誌，每日輪次	來源：對話日誌，每日輪次	出典：会話日誌、1日あたりの往復	출처: 대화 일지, 하루당 횟수	แหล่ง: บันทึกสนทนา รอบต่อวัน	Nguồn: nhật ký hội thoại, lượt mỗi ngày	Sumber: jurnal percakapan, giliran per hari	Sumber: jurnal perbualan, giliran sehari	Эх: ярианы тэмдэглэл, ээлж өдөрт
coverageCaption	来源：住宅图谱，设备占比	來源：住家圖譜，裝置占比	來源：屋企圖譜，裝置占比	出典：家のグラフ、機器の割合	출처: 집 그래프, 기기 비율	แหล่ง: กราฟบ้าน สัดส่วนอุปกรณ์	Nguồn: đồ thị nhà, phần thiết bị	Sumber: graf rumah, bagian perangkat	Sumber: graf rumah, bahagian peranti	Эх: гэрийн график, төхөөрөмжийн хувь
latency	阶段耗时	階段耗時	階段耗時	段階時間	단계 시간	เวลาขั้น	Thời giai đoạn	Waktu tahap	Masa peringkat	Шатны хугацаа
latencyCaption	来源：解析轨迹，微秒	來源：解析軌跡，微秒	來源：解析軌跡，微秒	出典：解析トレース、マイクロ秒	출처: 분석 추적, 마이크로초	แหล่ง: รอยแยก ไมโครวินาที	Nguồn: vết phân, micro giây	Sumber: jejak uraian, mikrodetik	Sumber: jejak huraian, mikrosaat	Эх: задлах мөр, микросекунд
unitsTurns	轮	輪	輪	往復	회	รอบ	lượt	giliran	giliran	ээлж
timeline	时间线	時間軸	時間軸	タイムライン	타임라인	เส้นเวลา	Dòng thời gian	Linimasa	Garis masa	Цагийн шугам
noConversations	还没有日志条目。	還沒有日誌條目。	未有日誌條目。	日誌の項目はまだありません。	일지 항목이 아직 없습니다.	ยังไม่มีรายการบันทึก	Chưa có mục nhật ký.	Belum ada entri jurnal.	Belum ada entri jurnal.	Тэмдэглэлийн мөр хараахан алга.
when	当	當	當	条件	조건	เมื่อ	Khi	Ketika	Apabila	Хэзээ
then	则	則	就	なら	그러면	แล้ว	Thì	Maka	Maka	Тэгвэл
priority	顺序（最先匹配的用户规则获胜）	順序（最先符合的使用者規則獲勝）	順序（最先符合嘅用戶規則贏）	順序（最初に一致したユーザールールが勝つ）	순서(먼저 맞는 사용자 규칙이 이김)	ลำดับ (กฎผู้ใช้ที่ตรงก่อนชนะ)	Thứ tự (luật người dùng khớp trước thắng)	Urutan (aturan pengguna yang cocok dulu menang)	Tertib (peraturan pengguna yang sepadan dulu menang)	Дараалал (эхний таарсан хэрэглэгчийн дүрэм ялна)
evaluator	策略求值器	原則評估器	策略評估器	方針評価器	정책 평가기	ตัวประเมินนโยบาย	Bộ đánh giá chính sách	Penilai kebijakan	Penilai dasar	Бодлогын үнэлэгч
bakeSpeech	生成变体	產生變體	產生變體	変異を生成	변형 생성	สร้างรูปแบบ	Tạo biến thể	Buat varian	Jana varian	Хувилбар үүсгэх
addRule	规则	規則	規則	ルール	규칙	กฎ	Quy tắc	Aturan	Peraturan	Дүрэм
noPolicies	还没有策略规则。	還沒有原則規則。	未有策略規則。	方針ルールはまだありません。	정책 규칙이 아직 없습니다.	ยังไม่มีกฎนโยบาย	Chưa có quy tắc chính sách.	Belum ada aturan kebijakan.	Belum ada peraturan dasar.	Бодлогын дүрэм хараахан алга.
compiledRisk	已编译风险	已編譯風險	已編譯風險	コンパイル済みリスク	컴파일된 위험	ความเสี่ยงที่คอมไพล์แล้ว	Rủi ro đã biên dịch	Risiko terkompilasi	Risiko terkompil	Эмхэтгэсэн эрсдэл
finalBand	档位	級距	級距	帯	대역	แถบ	Dải	Pita	Pita	Бүс
triggerFirst	HA 句子触发器优先，然后 Klar，然后已注册意图。	HA 句子觸發器優先，然後 Klar，然後已註冊意圖。	HA 句子觸發器先，然後 Klar，然後已註冊意圖。	HA の文トリガーが先、次に Klar、その後登録済み意図。	HA 문장 트리거가 먼저, 그다음 Klar, 그다음 등록된 의도.	ทริกเกอร์ประโยค HA ก่อน แล้ว Klar แล้วเจตนาที่ลงทะเบียน	Bộ kích câu HA trước, rồi Klar, rồi ý định đã đăng.	Pemicu kalimat HA dulu, lalu Klar, lalu niat terdaftar.	Pencetus ayat HA dulu, lalu Klar, lalu niat berdaftar.	HA өгүүлбэрийн триггер эхлээд, дараа Klar, дараа бүртгэлтэй зорилго.
discarded	已丢弃	已捨棄	丟咗	破棄	폐기됨	ทิ้งแล้ว	Đã bỏ	Dibuang	Dibuang	Хаясан
stageTokens	词元	詞元	詞元	トークン	토큰	โทเคน	Mã từ	Leksem	Leksem	Токен
stageBind	绑定	綁定	綁定	結び付け	연결	ผูก	Gắn	Ikat	Ikat	Холбох
stageRank	排序	排序	排序	順位	순위	จัดอันดับ	Xếp	Peringkat	Kedudukan	Эрэмбэ
stagePolicy	策略	原則	策略	方針	정책	นโยบาย	Chính sách	Kebijakan	Dasar	Бодлого
stageBand	档位	級距	級距	帯	대역	แถบ	Dải	Pita	Pita	Бүс
effectConfirm	确认	確認	確認	確認	확인	ยืนยัน	Xác nhận	Konfirmasi	Sahkan	Баталгаажуулах
effectBlock	阻止	阻擋	阻擋	阻止	차단	กันไว้	Chặn	Blokir	Sekat	Хориглох
effectAllow	允许	允許	允許	許可	허용	อนุญาต	Cho phép	Izinkan	Benarkan	Зөвшөөрөх
effectPreferEntity	优先实体	優先實體	優先實體	実体を優先	엔티티 우선	เลือกเอนทิตี	Ưu tiên thực thể	Utamakan entitas	Utamakan entiti	Нэгжийг илүүд
effectPreferArea	优先区域	優先區域	優先區域	領域を優先	영역 우선	เลือกพื้นที่	Ưu tiên khu	Utamakan kawasan	Utamakan kawasan	Бүсийг илүүд
effectReply	无意图回复	無意圖回覆	冇意圖回覆	意図なしで返答	의도 없이 답변	ตอบโดยไม่มีเจตนา	Trả lời không ý định	Balas tanpa niat	Balas tanpa niat	Зорилгогүй хариу
effectScript	脚本	指令碼	指令碼	スクリプト	스크립트	สคริปต์	Tập lệnh	Skrip	Skrip	Скрипт
effectTemplate	模板	範本	範本	テンプレート	템플릿	แม่แบบ	Mẫu	Templat	Templat	Загвар
effectLlm	LLM 提示词	LLM 提示	LLM 提示	LLM プロンプト	LLM 프롬프트	พรอมต์ LLM	Lời nhắc LLM	Perintah LLM	Prom LLM	LLM сануулга
payloadReply	回复文本	回覆文字	回覆文字	返答文	답변 텍스트	ข้อความตอบ	Văn trả lời	Teks balasan	Teks balasan	Хариу текст
payloadScript	脚本（script.good_night 或 good_night）	指令碼（script.good_night 或 good_night）	指令碼（script.good_night 或 good_night）	スクリプト（script.good_night または good_night）	스크립트(script.good_night 또는 good_night)	สคริปต์ (script.good_night หรือ good_night)	Tập lệnh (script.good_night hoặc good_night)	Skrip (script.good_night atau good_night)	Skrip (script.good_night atau good_night)	Скрипт (script.good_night эсвэл good_night)
payloadTemplate	Home Assistant 模板；{{ text }} 为用户原话	Home Assistant 範本；{{ text }} 為使用者原話	Home Assistant 範本；{{ text }} 係用戶原話	Home Assistant のテンプレート。{{ text }} は発話です	Home Assistant 템플릿. {{ text }}는 발화입니다	แม่แบบ Home Assistant; {{ text }} คือคำพูด	Mẫu Home Assistant; {{ text }} là lời nói	Templat Home Assistant; {{ text }} adalah ucapan	Templat Home Assistant; {{ text }} ialah ucapan	Home Assistant загвар; {{ text }} нь хэлсэн үг
payloadLlm	后备代理的系统提示词	後援代理的系統提示	後備代理嘅系統提示	予備エージェントのシステムプロンプト	예비 에이전트의 시스템 프롬프트	พรอมต์ระบบสำหรับเอเจนต์สำรอง	Lời nhắc hệ thống cho tác nhân dự phòng	Perintah sistem untuk agen cadangan	Prom sistem untuk ejen sandaran	Нөөц агентын системийн сануулга
whenPhrase	短语	語句	語句	フレーズ	구문	วลี	Cụm từ	Frasa	Frasa	Хэллэг
chatMode	闲聊	閒聊	傾偈	雑談	잡담	คุย	Trò chuyện	Obrolan	Sembang	Чат
variantPreview	语音变体	語音變體	語音變體	発話の変異	음성 변형	รูปแบบคำพูด	Biến thể lời nói	Varian ucapan	Varian pertuturan	Ярианы хувилбар
policies	策略	原則	策略	方針	정책	นโยบาย	Chính sách	Kebijakan	Dasar	Бодлого
routines	例程	例行	例行	ルーチン	루틴	รูทีน	Thói quen	Rutinitas	Rutin	Тогтмол
routineHint	说出名称即可启动 Home Assistant 脚本。晚安优先于问候。	說出名稱即可啟動 Home Assistant 指令碼。晚安優先於問候。	講出名就啟動 Home Assistant 指令碼。晚安贏問候。	名前を言うと Home Assistant のスクリプトが始まります。おやすみは挨拶より優先です。	말한 이름이 Home Assistant 스크립트를 시작합니다. 잘 자가 인사보다 앞섭니다.	ชื่อที่พูดจะเริ่มสคริปต์ Home Assistant ราตรีสวัสดิ์ชนะคำทักทาย	Tên nói ra khởi chạy tập lệnh Home Assistant. Chúc ngủ ngon thắng lời chào.	Nama yang diucapkan memulai skrip Home Assistant. Selamat malam menang atas sapaan.	Nama disebut memulakan skrip Home Assistant. Selamat malam menang ke atas ucapan.	Хэлсэн нэр Home Assistant скриптийг эхлүүлнэ. Сайхан амраарай мэндчилгээнээс түрүүлнэ.
routinePhraseHint	晚安	晚安	晚安	おやすみ	잘 자	ราตรีสวัสดิ์	Chúc ngủ ngon	Selamat malam	Selamat malam	Сайхан амраарай
addRoutine	添加例程	新增例行	新增例行	ルーチンを追加	루틴 추가	เพิ่มรูทีน	Thêm thói quen	Tambah rutinitas	Tambah rutin	Тогтмол нэмэх
noRoutines	还没有例程。	還沒有例行。	未有例行。	ルーチンはまだありません。	루틴이 아직 없습니다.	ยังไม่มีรูทีน	Chưa có thói quen.	Belum ada rutinitas.	Belum ada rutin.	Тогтмол хараахан алга.
routineInvalid	需要短语和 script.xxx。	需要語句和 script.xxx。	要語句同 script.xxx。	フレーズと script.xxx が必要です。	구문과 script.xxx가 필요합니다.	ต้องมีวลีและ script.xxx	Cần cụm từ và script.xxx.	Frasa dan script.xxx wajib.	Frasa dan script.xxx diperlukan.	Хэллэг ба script.xxx шаардлагатай.
lastTurn	上一轮	上一輪	上一輪	直前の往復	마지막 회	รอบล่าสุด	Lượt vừa rồi	Giliran terakhir	Giliran terakhir	Сүүлийн ээлж
heardIn	听到位置	聽到位置	聽到位置	聞いた場所	들린 위치	ได้ยินที่	Nghe tại	Didengar di	Didengar di	Сонссон газар
tryThese	你家房间里的五句	你家房間裡的五句	你屋企房間嘅五句	あなたの部屋での五文	당신 방의 다섯 문장	ห้าประโยคในห้องของคุณ	Năm câu trong phòng nhà bạn	Lima kalimat di ruangan Anda	Lima ayat di bilik anda	Таны өрөөний таван өгүүлбэр
tryTheseHint	点一句即可在实验室试。	點一句即可在實驗室試。	撳一句就喺實驗室試。	文をタップしてラボで試します。	문장을 눌러 랩에서 시험하세요.	แตะประโยคเพื่อลองในห้องทดลอง	Chạm một câu để thử trong thí nghiệm.	Ketuk kalimat untuk mencobanya di uji.	Ketuk ayat untuk mencubanya di makmal.	Өгүүлбэр дарж туршилтад туршина уу.
anyRoom	无卫星	無衛星	冇衛星	衛星なし	위성 없음	ไม่มีดาวเทียม	Không vệ tinh	Tanpa satelit	Tiada satelit	Хиймэл дагуулгүй
personalityHa	在 Home Assistant → Klar NLU → 人格 中设置人格。	在 Home Assistant → Klar NLU → 人格 中設定人格。	喺 Home Assistant → Klar NLU → 人格 設定人格。	人格は Home Assistant → Klar NLU → 人格 で設定します。	성격은 Home Assistant → Klar NLU → 성격에서 설정하세요.	ตั้งบุคลิกใน Home Assistant → Klar NLU → บุคลิก	Đặt tính cách trong Home Assistant → Klar NLU → Tính cách.	Atur kepribadian di Home Assistant → Klar NLU → Kepribadian.	Tetapkan personaliti dalam Home Assistant → Klar NLU → Personaliti.	Зан чанарыг Home Assistant → Klar NLU → Зан чанар-д тохируулна.
"""

PACKS = parse_table(CODES, TABLE)
