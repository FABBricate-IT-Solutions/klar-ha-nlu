"""Leftover Assist calendar lexemes (Latin-script packs)."""

from __future__ import annotations

LATIN_MORE = {
    "cy": (["calendr", "apwyntiad"], ["nesaf"], ["ychwanegu"], ["heddiw"], ["yfory"], ["am"], ["ailgychwyn"], "beth sydd ar y calendr", "ychwanegu deintydd yfory am 15 calendr", "Do ddeallais i ddim."),
    "et": (["kalender", "kohtumine"], ["tulevased"], ["lisa"], ["taena"], ["homme"], ["kell"], ["jatka"], "mis on kalendris", "lisa hambaarst homme kell 15 kalender", "Ma ei saanud aru."),
    "eu": (["egutegi", "hitzordu"], ["hurrengo"], ["gehitu"], ["gaur"], ["bihar"], ["etan"], ["jarraitu"], "zer dago egutegian", "gehitu dentista bihar 15 egutegi", "Ez dut ulertu."),
    "ga": (["feilire", "coinne"], ["ata"], ["cuir"], ["inniu"], ["amarach"], ["ag"], ["atchuir"], "cad ata ar an fheilire", "cuir fiacloir amarach ag 15 feilire", "Nior thuig me."),
    "gl": (["calendario", "cita"], ["proximos"], ["engade"], ["hoxe"], ["manha"], ["as"], ["continua"], "que hai no calendario", "engade dentista manha as 15 calendario", "Non o entendin."),
    "is": (["dagatal", "fundur"], ["naestu"], ["baettu"], ["idag"], ["amorgun"], ["kl"], ["halda"], "hvad er a dagatalinu", "baettu tannlaekni amorgun kl 15 dagatal", "Eg skildi thetta ekki."),
    "lb": (["kalenner", "termin"], ["kommen"], ["derbai"], ["haut"], ["moien"], ["um"], ["virun"], "wat steet am kalenner", "termin zahnarzt moien um 15 kalenner", "Dat hunn ech net verstan."),
    "kw": (["devis", "omwelyans"], ["nessa"], ["kewgh"], ["hedhyw"], ["avorow"], ["dhe"], ["dastin"], "pyth usi yn devis", "kewgh dentydh avorow dhe 15 devis", "Ny gonvedhis."),
    "lt": (["kalendorius", "susitikimas"], ["ateinantys"], ["prideti"], ["siandien"], ["rytoj"], ["val"], ["testi"], "kas kalendoriuje", "prideti odontologa rytoj 15 kalendorius", "Nesupratau."),
    "lv": (["kalendars", "tiksanos"], ["nakamie"], ["pievieno"], ["sodien"], ["rit"], ["plkst"], ["turpini"], "kas ir kalendara", "pievieno zobarstu rit 15 kalendars", "Nesapratu."),
    "id": (["kalender", "janji"], ["mendatang"], ["tambah"], ["hariini"], ["besok"], ["jam"], ["lanjut"], "apa di kalender", "tambah dokter besok jam 15 kalender", "Saya tidak mengerti."),
    "ms": (["kalendar", "temujanji"], ["akan"], ["tambah"], ["hariini"], ["esok"], ["pukul"], ["sambung"], "apa dalam kalendar", "tambah doktor esok pukul 15 kalendar", "Saya tidak faham."),
    "sw": (["kalenda", "miadi"], ["zijazo"], ["ongeza"], ["leo"], ["kesho"], ["saa"], ["endelea"], "nini kwenye kalenda", "ongeza daktari kesho saa 15 kalenda", "Sikuelewa."),
    "vi": (["lich", "cuochen"], ["saptoi"], ["them"], ["homnay"], ["ngaymai"], ["luc"], ["tiep"], "lich co gi", "them nha si ngaymai 15 lich", "Toi khong hieu."),
    "hi": (["calendar", "kalendar", "mulakat", "कैलेंडर"], ["aane"], ["jodo"], ["aaj"], ["kal"], ["baje"], ["jari"], "calendar me kya hai", "jodo dentist kal 15 calendar", "Samajh nahi aaya."),
    "bn": (["kalendar", "apointment"], ["asanna"], ["jogo"], ["aj"], ["kal"], ["tay"], ["chalu"], "kalendar e ki", "jogo dentist kal 15 kalendar", "Bujhte parini."),
    "gu": (["calendar", "mulakat"], ["aavnar"], ["umero"], ["aaje"], ["kale"], ["vage"], ["chalu"], "calendar ma su", "umero dentist kale 15 calendar", "Samajayu nahi."),
    "kn": (["calendar", "bheti"], ["baruvud"], ["serisu"], ["indu"], ["nale"], ["gante"], ["munde"], "calendar alli enu", "serisu dentist nale 15 calendar", "Artha agalilla."),
    "ml": (["calendar", "kootayma"], ["varunna"], ["chertu"], ["innu"], ["nale"], ["mani"], ["thudaru"], "calendar il enthu", "chertu dentist nale 15 calendar", "Manasilayilla."),
    "mr": (["calendar", "bhet"], ["yenare"], ["bhar"], ["aaj"], ["udya"], ["vajta"], ["chalu"], "calendar madhe kay", "bhar dentist udya 15 calendar", "Kalale nahi."),
    "ta": (["calendar", "sandippu"], ["varum"], ["seru"], ["indru"], ["naalai"], ["mani"], ["thodar"], "calendar la enna", "seru dentist naalai 15 calendar", "Puriyavillai."),
    "te": (["calendar", "kalayika"], ["vacce"], ["ekku"], ["eeroju"], ["repu"], ["gantala"], ["kotesu"], "calendar lo emi", "ekku dentist repu 15 calendar", "Ardham kaledu."),
    "pa": (["calendar", "mulakat"], ["aan"], ["pao"], ["ajj"], ["kal"], ["vaje"], ["jari"], "calendar vich ki", "pao dentist kal 15 calendar", "Samajh nahi aya."),
    "ne": (["calendar", "bhet"], ["aune"], ["thap"], ["aaja"], ["bholi"], ["baje"], ["jari"], "calendar ma ke", "thap dentist bholi 15 calendar", "Bujhina."),
    "hy": (["oracuyc", "handipum"], ["galik"], ["avelacnel"], ["aysor"], ["vaghy"], ["zham"], ["sharunakel"], "inch ka oracuycum", "avelacnel atamnabuzh vaghy 15 oracuyc", "Chhasetskatsi."),
    "ka": (["kalendari", "shekhvedra"], ["momaval"], ["daamate"], ["dghes"], ["khval"], ["saati"], ["ganagrdze"], "ra aris kalendarshi", "daamate stomatologi khval 15 kalendari", "Ver gavige."),
    "mn": (["tsag", "uulzalt"], ["ireh"], ["nem"], ["onoodor"], ["margash"], ["tsag"], ["urgejil"], "tsagand yu baina", "nem shudny margash 15 tsag", "Oilgosongui."),
}


def merge_more(calendar: dict, row, speech) -> None:
    for code, item in LATIN_MORE.items():
        nouns, query, create, today, tomorrow, when, resume, list_s, create_s, unknown = item
        calendar[code] = row(
            nouns,
            query,
            create,
            today,
            tomorrow,
            when,
            resume,
            list_s,
            create_s,
            speech(unknown, "{items}", unknown, unknown, "{summary} {when}.", unknown, unknown, unknown, unknown),
        )
