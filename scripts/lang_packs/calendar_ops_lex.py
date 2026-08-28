"""Delete/move verbs, smokes, and speech for every Assist pack."""

from __future__ import annotations

# delete, move, delete_smoke, move_smoke, deleted, moved, which, no_uid
OPS: dict[str, tuple[list[str], list[str], str, str, str, str, str, str]] = {
    "de": (["loesch", "lösch", "streich"], ["verschieb", "verleg"], "loesch zahnarzt kalender", "verschieb zahnarzt morgen um 16 kalender", "Termin gelöscht.", "{summary} {when}.", "Welcher Termin?", "Dieser Termin hat keine Kennung."),
    "en": (["delete", "cancel", "remove"], ["move", "reschedule"], "delete dentist calendar", "move dentist tomorrow at 4 calendar", "Event deleted.", "{summary} {when}.", "Which event?", "That event has no identifier."),
    "fr": (["supprime", "annule"], ["deplace", "reporte"], "supprime dentiste calendrier", "deplace dentiste demain a 16 calendrier", "Rendez-vous supprime.", "{summary} {when}.", "Quel rendez-vous ?", "Pas d identifiant."),
    "nl": (["verwijder", "wis"], ["verplaats", "verschuif"], "verwijder tandarts kalender", "verplaats tandarts morgen om 16 kalender", "Afspraak verwijderd.", "{summary} {when}.", "Welke afspraak?", "Geen kenmerk."),
    "es": (["borra", "elimina"], ["mueve", "cambia"], "borra dentista calendario", "mueve dentista manana a 16 calendario", "Cita borrada.", "{summary} {when}.", "Que cita?", "Sin identificador."),
    "it": (["cancella", "elimina"], ["sposta", "sposta"], "cancella dentista calendario", "sposta dentista domani alle 16 calendario", "Appuntamento cancellato.", "{summary} {when}.", "Quale appuntamento?", "Nessun identificativo."),
    "pt": (["apaga", "remove"], ["move", "adia"], "apaga dentista calendario", "move dentista amanha as 16 calendario", "Evento apagado.", "{summary} {when}.", "Qual evento?", "Sem identificador."),
    "ca": (["esborra"], ["mou"], "esborra dentista calendari", "mou dentista dema a 16 calendari", "Cita esborrada.", "{summary} {when}.", "Quina cita?", "Sense identificador."),
    "ro": (["sterge"], ["muta"], "sterge dentist calendar", "muta dentist maine la 16 calendar", "Eveniment sters.", "{summary} {when}.", "Care eveniment?", "Fara identificator."),
    "da": (["slet"], ["flyt"], "slet tandlaege kalender", "flyt tandlaege imorgen kl 16 kalender", "Aftale slettet.", "{summary} {when}.", "Hvilken aftale?", "Intet id."),
    "nb": (["slett"], ["flytt"], "slett tannlege kalender", "flytt tannlege imorgen kl 16 kalender", "Avtale slettet.", "{summary} {when}.", "Hvilken avtale?", "Ingen id."),
    "sv": (["radera"], ["flytta"], "radera tandlakare kalender", "flytta tandlakare imorgon kl 16 kalender", "Mote raderat.", "{summary} {when}.", "Vilket mote?", "Inget id."),
    "fi": (["poista"], ["siirra"], "poista hammaslaakari kalenteri", "siirra hammaslaakari huomenna klo 16 kalenteri", "Tapahtuma poistettu.", "{summary} {when}.", "Mika tapahtuma?", "Ei tunnisteita."),
    "de-CH": (["lösch", "strich"], ["verschieb"], "lösch zahnarzt kalender", "verschieb zahnarzt morn um 16 kalender", "Termin glöscht.", "{summary} {when}.", "Wele Termin?", "Kei Kennig."),
    "de-AT": (["loesch", "streich"], ["verschieb"], "loesch zahnarzt kalender", "verschieb zahnarzt morgen um 16 kalender", "Termin gelöscht.", "{summary} {when}.", "Welcher Termin?", "Keine Kennung."),
    "en-GB": (["delete", "cancel"], ["move"], "delete dentist calendar", "move dentist tomorrow at 4 calendar", "Event deleted.", "{summary} {when}.", "Which event?", "No identifier."),
    "pt-BR": (["apaga"], ["move"], "apaga dentista calendario", "move dentista amanha as 16 calendario", "Evento apagado.", "{summary} {when}.", "Qual evento?", "Sem id."),
    "af": (["verwyder"], ["skuif"], "verwyder tandarts kalender", "skuif tandarts more om 16 kalender", "Afspraak verwyder.", "{summary} {when}.", "Watter afspraak?", "Geen id."),
    "cs": (["smaz"], ["presun"], "smaz zubare kalendar", "presun zubare zitra v 16 kalendar", "Smazano.", "{summary} {when}.", "Ktera udalost?", "Bez id."),
    "sk": (["zmaz"], ["presun"], "zmaz zubara kalendar", "presun zubara zajtra o 16 kalendar", "Zmazane.", "{summary} {when}.", "Ktora udalost?", "Bez id."),
    "pl": (["usun"], ["przenies"], "usun dentyste kalendarz", "przenies dentyste jutro o 16 kalendarz", "Usunieto.", "{summary} {when}.", "Ktore wydarzenie?", "Bez id."),
    "hu": (["torol"], ["athelyez"], "torol fogorvos naptar", "athelyez fogorvos holnap 16 naptar", "Torolve.", "{summary} {when}.", "Melyik esemeny?", "Nincs id."),
    "hr": (["obrisi"], ["premjesti"], "obrisi zubara kalendar", "premjesti zubara sutra u 16 kalendar", "Obrisano.", "{summary} {when}.", "Koji dogadaj?", "Nema id."),
    "sl": (["izbrisi"], ["premakni"], "izbrisi zobozdravnika koledar", "premakni zobozdravnika jutri ob 16 koledar", "Izbrisano.", "{summary} {when}.", "Kateri dogodek?", "Ni id."),
    "bg": (["изтрий"], ["премести"], "изтрий зъболекар календар", "премести зъболекар утре в 16 календар", "Изтрито.", "{summary} {when}.", "Кое събитие?", "Няма id."),
    "el": (["διαγραψε"], ["μεταφερε"], "διαγραψε οδοντιατρο ημερολογιο", "μεταφερε οδοντιατρο αυριο στις 16 ημερολογιο", "Διαγραφηκε.", "{summary} {when}.", "Ποιο γεγονος?", "Χωρις id."),
    "sr": (["обриши"], ["премести"], "обриши зубара календар", "премести зубара сутра у 16 календар", "Обрисано.", "{summary} {when}.", "Који догађај?", "Нема id."),
    "sr-Latn": (["obrisi"], ["premesti"], "obrisi zubara kalendar", "premesti zubara sutra u 16 kalendar", "Obrisano.", "{summary} {when}.", "Koji dogadjaj?", "Nema id."),
    "uk": (["видали"], ["перенеси"], "видали стоматолога календар", "перенеси стоматолога завтра о 16 календар", "Видалено.", "{summary} {when}.", "Яка подія?", "Немає id."),
    "zh-CN": (["shanchu"], ["yidong"], "shanchu yasheng rili", "yidong yasheng mingtian 16 rili", "已删除。", "{summary} {when}。", "哪个日程？", "没有编号。"),
    "zh-TW": (["shanchu"], ["yidong"], "shanchu yisheng rili", "yidong yisheng mingtian 16 rili", "已刪除。", "{summary} {when}。", "哪個行程？", "沒有編號。"),
    "zh-HK": (["shanchu"], ["yidong"], "shanchu yisang rili", "yidong yisang mingtian 16 rili", "刪咗。", "{summary} {when}。", "邊個行程？", "冇編號。"),
    "ar": (["احذف"], ["انقل"], "احذف طبيب تقويم", "انقل طبيب غدا الساعة 16 تقويم", "تم الحذف.", "{summary} {when}.", "أي موعد؟", "بلا معرف."),
    "he": (["מחק"], ["העבר"], "מחק רופא יומן", "העבר רופא מחר ב 16 יומן", "נמחק.", "{summary} {when}.", "איזה אירוע?", "אין מזהה."),
    "fa": (["حذف"], ["جابجا"], "حذف دندانپزشک تقویم", "جابجا دندانپزشک فردا ساعت 16 تقویم", "حذف شد.", "{summary} {when}.", "کدام رویداد؟", "بدون شناسه."),
    "ur": (["حذف"], ["منتقل"], "حذف دانت کیلنڈر", "منتقل دانت کل 16 کیلنڈر", "حذف ہو گیا۔", "{summary} {when}.", "کون سی تقریب؟", "کوئی شناخت نہیں۔"),
    "tr": (["sil"], ["tasi"], "sil disci takvim", "tasi disci yarin saat 16 takvim", "Silindi.", "{summary} {when}.", "Hangi etkinlik?", "Kimlik yok."),
    "th": (["ลบ"], ["ย้าย"], "ลบ หมอ ปฏิทิน", "ย้าย หมอ พรุ่งนี้ 16 ปฏิทิน", "ลบแล้ว", "{summary} {when}", "นัดไหน", "ไม่มีรหัส"),
    "ko": (["sakje"], ["omgyeo"], "sakje chigwa dallyeok", "omgyeo chigwa naeil 16 dallyeok", "삭제했어요.", "{summary} {when}.", "어떤 일정?", "식별자가 없어요."),
    "ja": (["sakujo"], ["ido"], "sakujo haisha karendaa", "ido haisha ashita 16 karendaa", "削除しました。", "{summary} {when}。", "どの予定？", "識別子がありません。"),
    "cy": (["dileu"], ["symud"], "dileu deintydd calendr", "symud deintydd yfory am 16 calendr", "Wedi dileu.", "{summary} {when}.", "Pa ddigwyddiad?", "Dim id."),
    "et": (["kustuta"], ["liiguta"], "kustuta hambaarst kalender", "liiguta hambaarst homme kell 16 kalender", "Kustutatud.", "{summary} {when}.", "Milline sundmus?", "Pole id."),
    "eu": (["ezabatu"], ["mugitu"], "ezabatu dentista egutegi", "mugitu dentista bihar 16 egutegi", "Ezabatuta.", "{summary} {when}.", "Zein hitzordu?", "Id ez."),
    "ga": (["scrios"], ["bog"], "scrios fiacloir feilire", "bog fiacloir amarach ag 16 feilire", "Scriosta.", "{summary} {when}.", "Ce imeacht?", "Gan id."),
    "gl": (["borra"], ["move"], "borra dentista calendario", "move dentista manha as 16 calendario", "Borrado.", "{summary} {when}.", "Que cita?", "Sen id."),
    "is": (["eyda"], ["faera"], "eyda tannlaekni dagatal", "faera tannlaekni amorgun kl 16 dagatal", "Eytt.", "{summary} {when}.", "Hvad atburdur?", "Ekkert id."),
    "lb": (["lasch"], ["rekleck"], "lasch zahnarzt kalenner", "rekleck zahnarzt moien um 16 kalenner", "Geläscht.", "{summary} {when}.", "Wee Termin?", "Keng id."),
    "kw": (["dile"], ["gwaya"], "dile dentydh devis", "gwaya dentydh avorow dhe 16 devis", "Diles.", "{summary} {when}.", "Py hwarvos?", "Heb id."),
    "lt": (["istrink"], ["perkelk"], "istrink odontologa kalendorius", "perkelk odontologa rytoj 16 kalendorius", "Istrinta.", "{summary} {when}.", "Kuris ivykis?", "Nera id."),
    "lv": (["dzes"], ["parvietot"], "dzes zobarstu kalendars", "parvietot zobarstu rit 16 kalendars", "Dzests.", "{summary} {when}.", "Kurss notikums?", "Nav id."),
    "id": (["hapus"], ["pindah"], "hapus dokter kalender", "pindah dokter besok jam 16 kalender", "Dihapus.", "{summary} {when}.", "Acara mana?", "Tidak ada id."),
    "ms": (["padam"], ["alih"], "padam doktor kalendar", "alih doktor esok pukul 16 kalendar", "Dipadam.", "{summary} {when}.", "Acara yang mana?", "Tiada id."),
    "sw": (["futa"], ["hamisha"], "futa daktari kalenda", "hamisha daktari kesho saa 16 kalenda", "Imefutwa.", "{summary} {when}.", "Tukio lipi?", "Hakuna id."),
    "vi": (["xoa"], ["chuyen"], "xoa nha si lich", "chuyen nha si ngaymai 16 lich", "Da xoa.", "{summary} {when}.", "Su kien nao?", "Khong id."),
    "hi": (["hatao"], ["sarkao"], "hatao dentist calendar", "sarkao dentist kal 16 calendar", "Hata diya.", "{summary} {when}.", "Kaun sa event?", "Id nahi."),
    "bn": (["much"], ["sorao"], "much dentist kalendar", "sorao dentist kal 16 kalendar", "Muchechi.", "{summary} {when}.", "Kon event?", "Id nai."),
    "gu": (["kadhi"], ["khalav"], "kadhi dentist calendar", "khalav dentist kale 16 calendar", "Kadi didhu.", "{summary} {when}.", "Kyo event?", "Id nathi."),
    "kn": (["anisu"], ["sarisu"], "anisu dentist calendar", "sarisu dentist nale 16 calendar", "Aniside.", "{summary} {when}.", "Yava event?", "Id illa."),
    "ml": (["maattu"], ["nivarthu"], "maattu dentist calendar", "nivarthu dentist nale 16 calendar", "Maatti.", "{summary} {when}.", "Ethu event?", "Id illa."),
    "mr": (["kad"], ["halav"], "kad dentist calendar", "halav dentist udya 16 calendar", "Kadale.", "{summary} {when}.", "Kontya event?", "Id nahi."),
    "ta": (["neekku"], ["maatru"], "neekku dentist calendar", "maatru dentist naalai 16 calendar", "Neekkappattathu.", "{summary} {when}.", "Endha event?", "Id illai."),
    "te": (["teeyu"], ["marupu"], "teeyu dentist calendar", "marupu dentist repu 16 calendar", "Teeyadam.", "{summary} {when}.", "Edi event?", "Id ledu."),
    "pa": (["hatao"], ["saraka"], "hatao dentist calendar", "saraka dentist kal 16 calendar", "Hata ditta.", "{summary} {when}.", "Kehda event?", "Id nahi."),
    "ne": (["hatau"], ["saru"], "hatau dentist calendar", "saru dentist bholi 16 calendar", "Hatayo.", "{summary} {when}.", "Kun event?", "Id chaina."),
    "hy": (["jnjel"], ["teghapoxel"], "jnjel atamnabuzh oracuyc", "teghapoxel atamnabuzh vaghy 16 oracuyc", "Jnjel e.", "{summary} {when}.", "Vor iradarcutyun?", "Id chka."),
    "ka": (["tsashala"], ["gadauqvan"], "tsashala stomatologi kalendari", "gadauqvan stomatologi khval 16 kalendari", "Tsashlilia.", "{summary} {when}.", "Romeli movlena?", "Id ar aris."),
    "mn": (["ustga"], ["shilj"], "ustga shudny tsag", "shilj shudny margash 16 tsag", "Ustgasan.", "{summary} {when}.", "Ali uulzalt?", "Id baihgui."),
}


def merge_ops(calendar: dict) -> None:
    for code, row in calendar.items():
        item = OPS.get(code) or OPS["en"]
        delete, move, delete_s, move_s, deleted, moved, which, no_uid = item
        row["delete"] = delete
        row["move"] = move
        row["delete_smoke"] = delete_s
        row["move_smoke"] = move_s
        speech = row.setdefault("speech", {})
        speech["calendar_deleted"] = deleted
        speech["calendar_moved"] = moved
        speech["calendar_which"] = which
        speech["calendar_no_uid"] = no_uid
