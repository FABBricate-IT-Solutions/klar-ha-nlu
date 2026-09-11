"""Dashboard LLM chrome for East and Southeast Asian Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["zh-CN", "zh-TW", "zh-HK", "ja", "ko", "th", "vi", "id", "ms", "mn"]

TABLE = """
llmCalls	LLM 调用	LLM 呼叫	LLM 呼叫	LLM 呼び出し	LLM 호출	การเรียก LLM	Lần gọi LLM	Panggilan LLM	Panggilan LLM	LLM дуудлага
llmCallsCaption	来源：引擎 LLM，最近 24 小时	來源：引擎 LLM，最近 24 小時	來源：引擎 LLM，最近 24 小時	出典: エンジン LLM、直近 24 時間	출처: 엔진 LLM, 최근 24시간	แหล่งที่มา: LLM ของเอนจิน 24 ชั่วโมงล่าสุด	Nguồn: LLM của động cơ, 24 giờ qua	Sumber: LLM mesin, 24 jam terakhir	Sumber: LLM enjin, 24 jam terakhir	Эх сурвалж: хөдөлгүүрийн LLM, сүүлийн 24 цаг
llmKindRefine	润色	潤飾	潤飾	整え	다듬기	ขัดเกลา	tinh chỉnh	perhalus	perhalusi	цэвэршүүлэлт
llmKindAssist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist
llmKindChat	闲聊	閒聊	閒聊	雑談	잡담	คุยเล่น	trò chuyện	obrolan	sembang	яриа
llmErrors	错误	錯誤	錯誤	エラー	오류	ข้อผิดพลาด	lỗi	galat	ralat	алдаа
llmAcceptRate	接受率	接受率	接受率	採用率	수락률	อัตราการรับ	tỷ lệ chấp nhận	tingkat terima	kadar terima	хүлээн авах хувь
llmLatency	LLM 延迟	LLM 延遲	LLM 延遲	LLM 遅延	LLM 지연	ความหน่วง LLM	Độ trễ LLM	Latensi LLM	Kependaman LLM	LLM саатал
llmLatencyCaption	来源：引擎 LLM，滚动窗口	來源：引擎 LLM，滾動視窗	來源：引擎 LLM，滾動視窗	出典: エンジン LLM、ローリング窓	출처: 엔진 LLM, 롤링 창	แหล่งที่มา: LLM ของเอนจิน หน้าต่างเลื่อน	Nguồn: LLM của động cơ, cửa sổ trượt	Sumber: LLM mesin, jendela bergulir	Sumber: LLM enjin, tetingkap bergolek	Эх сурвалж: хөдөлгүүрийн LLM, гулсах цонх
llmTokens	词元	詞元	詞元	トークン	토큰	โทเค็น	token	token	token	токен
llmTokensCaption	来源：上游用量，模型提供时才显示	來源：上游用量，模型提供時才顯示	來源：上游用量，模型提供先顯示	出典: 上流の使用量。モデルが返すときだけ	출처: 업스트림 사용량, 모델이 보낼 때만	แหล่งที่มา: ปริมาณต้นน้ำ เมื่อโมเดลส่งมา	Nguồn: mức dùng phía trên, khi mô hình gửi	Sumber: pemakaian hulu, saat model mengirimnya	Sumber: penggunaan hulu, bila model menghantarnya	Эх сурвалж: дээд урсгалын хэрэглээ, загвар илгээхэд
llmNoCalls	还没有 LLM 调用。	還沒有 LLM 呼叫。	未有 LLM 呼叫。	まだ LLM 呼び出しはありません。	아직 LLM 호출이 없습니다.	ยังไม่มีการเรียก LLM	Chưa có lần gọi LLM.	Belum ada panggilan LLM.	Belum ada panggilan LLM.	LLM дуудлага хараахан алга.
labThisTurn	这一轮	這一輪	呢一輪	このターン	이번 턴	รอบนี้	Lượt này	Giliran ini	Giliran ini	Энэ ээлж
"""

PACKS = parse_rows(CODES, TABLE)
