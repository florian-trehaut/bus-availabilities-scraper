use std::collections::HashMap;
use std::sync::LazyLock;

pub static ROUTE_NAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // Area 1
        ("新宿～富士五湖線", "Shinjuku - Fuji Five Lakes"),
        ("新宿～甲府線", "Shinjuku - Kofu"),
        (
            "新宿～身延・南アルプス市八田線",
            "Shinjuku - Minobu/South Alps Hatta",
        ),
        (
            "新宿～さがみ湖イルミリオン線",
            "Shinjuku - Sagamiko Illumillion",
        ),
        ("新宿～諏訪・岡谷・茅野線", "Shinjuku - Suwa/Okaya/Chino"),
        ("新宿～伊那・飯田線", "Shinjuku - Ina/Iida"),
        ("新宿～松本線", "Shinjuku - Matsumoto"),
        ("新宿・池袋～長野線", "Shinjuku/Ikebukuro - Nagano"),
        ("新宿～白馬線", "Shinjuku - Hakuba"),
        (
            "新宿～塩尻・木曽福島線",
            "Shinjuku - Shiojiri/Kiso-Fukushima",
        ),
        ("成田空港～軽井沢線", "Narita Airport - Karuizawa"),
        (
            "新宿～上高地線（さわやか信州号）",
            "Shinjuku - Kamikochi (Sawayaka Shinshu)",
        ),
        ("新宿～飛騨高山線", "Shinjuku - Hida Takayama"),
        ("新宿～名古屋線", "Shinjuku - Nagoya"),
        ("岐阜～新宿線", "Gifu - Shinjuku"),
        (
            "新宿・渋谷～三島・沼津線",
            "Shinjuku/Shibuya - Mishima/Numazu",
        ),
        (
            "新宿・渋谷～清水・静岡線",
            "Shinjuku/Shibuya - Shimizu/Shizuoka",
        ),
        ("新宿・渋谷～浜松線", "Shinjuku/Shibuya - Hamamatsu"),
        ("新宿～サマーランド線", "Shinjuku - Summerland"),
        (
            "新宿・渋谷～仙台・石巻線",
            "Shinjuku/Shibuya - Sendai/Ishinomaki",
        ),
        (
            "東京・新宿～青森線（ノクターン・ネオ号）",
            "Tokyo/Shinjuku - Aomori (Nocturne Neo)",
        ),
        (
            "新宿・大宮～八戸・三沢・むつ線（しもきた号）",
            "Shinjuku/Omiya - Hachinohe/Misawa/Mutsu (Shimokita)",
        ),
        (
            "新宿・渋谷～大阪（阪急梅田）・ＵＳＪ線",
            "Shinjuku/Shibuya - Osaka (Hankyu Umeda)/USJ",
        ),
        (
            "船橋・新宿・東京～京都・大阪線（アウルライナー）",
            "Funabashi/Shinjuku/Tokyo - Kyoto/Osaka (Owl Liner)",
        ),
        ("新宿～大阪線　ツインクル", "Shinjuku - Osaka (Twinkle)"),
        ("新宿～大阪線　カジュアル", "Shinjuku - Osaka (Casual)"),
        ("新宿・渋谷～神戸姫路線", "Shinjuku/Shibuya - Kobe/Himeji"),
        (
            "東京・新宿・横浜～高松・丸亀線",
            "Tokyo/Shinjuku/Yokohama - Takamatsu/Marugame",
        ),
        (
            "東京・新宿～徳島・阿南線（マイ・エクスプレス号）",
            "Tokyo/Shinjuku - Tokushima/Anan (My Express)",
        ),
        ("新宿・横浜～松山線", "Shinjuku/Yokohama - Matsuyama"),
        // Area 2
        ("名古屋～福岡線", "Nagoya - Fukuoka"),
        ("名古屋～岡山線", "Nagoya - Okayama"),
        ("名古屋～仙台線", "Nagoya - Sendai"),
        ("名古屋～宇都宮・郡山線", "Nagoya - Utsunomiya/Koriyama"),
        ("竜王・甲府～名古屋線", "Ryuo/Kofu - Nagoya"),
        ("名古屋～富士五湖線", "Nagoya - Fuji Five Lakes"),
        ("名古屋～上高地線", "Nagoya - Kamikochi"),
        ("名古屋～高山線", "Nagoya - Takayama"),
        ("名古屋～白川郷・金沢線", "Nagoya - Shirakawa-go/Kanazawa"),
        ("名古屋～金沢線", "Nagoya - Kanazawa"),
        ("名古屋～郡上ひるがの線", "Nagoya - Gujo Hirugano"),
        ("名古屋～富山線", "Nagoya - Toyama"),
        ("名古屋～高岡・砺波線", "Nagoya - Takaoka/Tonami"),
        ("名古屋～福井線", "Nagoya - Fukui"),
        ("名古屋～松本線", "Nagoya - Matsumoto"),
        ("名古屋～伊那・箕輪線", "Nagoya - Ina/Minowa"),
        ("名古屋～飯田線", "Nagoya - Iida"),
        ("名古屋～馬籠・妻籠線", "Nagoya - Magome/Tsumago"),
        // Area 3
        (
            "羽田～調布・若葉台・国分寺・武蔵小金井線",
            "Haneda - Chofu/Wakabadai/Kokubunji/Musashi-Koganei",
        ),
        ("羽田多摩センター線", "Haneda - Tama Center"),
        ("羽田八王子線", "Haneda - Hachioji"),
        (
            "羽田～河辺・羽村・福生・秋川線",
            "Haneda - Kawabe/Hamura/Fussa/Akigawa",
        ),
        (
            "羽田・東京（八重洲）～草津線（温泉アクセスライナー草津）",
            "Haneda/Tokyo (Yaesu) - Kusatsu (Onsen Access Liner)",
        ),
    ])
});

pub static STATION_NAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // === SHINJUKU / TOKYO AREA TERMINALS ===
        (
            "バスタ新宿（南口）",
            "Shinjuku Expressway Bus Terminal (South Exit)",
        ),
        (
            "新宿西口臨時便２６番のりば",
            "Shinjuku West Exit Temporary Platform 26",
        ),
        ("新宿西口２５番のりば", "Shinjuku West Exit Platform 25"),
        ("東京駅八重洲南口", "Tokyo Station Yaesu South Exit"),
        ("東京駅鉄鋼ビル", "Tokyo Station Tekko Building"),
        (
            "渋谷マークシティバスターミナル",
            "Shibuya Mark City Bus Terminal",
        ),
        ("池袋駅東口", "Ikebukuro Station East Exit"),
        (
            "池袋サンシャインバスターミナル",
            "Ikebukuro Sunshine Bus Terminal",
        ),
        ("横浜駅ＹＣＡＴ", "Yokohama Station YCAT"),
        ("品川バスターミナル", "Shinagawa Bus Terminal"),
        ("大宮駅西口", "Omiya Station West Exit"),
        ("練馬区役所前", "Nerima Ward Office"),
        ("川越的場バスストップ", "Kawagoe Matoba Bus Stop"),
        ("所沢駅東口", "Tokorozawa Station East Exit"),
        ("二子玉川駅", "Futako-Tamagawa Station"),
        ("たまプラーザ駅", "Tama Plaza Station"),
        ("船橋駅北口", "Funabashi Station North Exit"),
        ("東京ディズニーランド", "Tokyo Disneyland"),
        ("東京ディズニーシー", "Tokyo DisneySea"),
        // === CHUO EXPRESSWAY STOPS ===
        ("中央道三鷹", "Chuo Expressway Mitaka"),
        ("中央道深大寺", "Chuo Expressway Jindaiji"),
        ("中央道府中", "Chuo Expressway Fuchu"),
        ("中央道日野", "Chuo Expressway Hino"),
        ("中央道八王子", "Chuo Expressway Hachioji"),
        ("中央道石川ＰＡ", "Chuo Expressway Ishikawa PA"),
        ("中央道相模湖", "Chuo Expressway Sagamiko"),
        ("中央道上野原", "Chuo Expressway Uenohara"),
        ("中央道小形山", "Chuo Expressway Ogatayama"),
        ("中央道大月", "Chuo Expressway Otsuki"),
        ("中央道都留", "Chuo Expressway Tsuru"),
        ("中央道西桂", "Chuo Expressway Nishikatsura"),
        ("中央道下吉田", "Chuo Expressway Shimoyoshida"),
        // === FUJI FIVE LAKES AREA ===
        ("富士急ハイランド", "Fuji-Q Highland"),
        ("河口湖駅", "Kawaguchiko Station"),
        ("富士山駅", "Fujisan Station"),
        ("富士吉田市役所入口", "Fujiyoshida City Hall Entrance"),
        ("山中湖　旭日丘", "Yamanakako Asahigaoka"),
        ("山中湖　御殿場口", "Yamanakako Gotemba Exit"),
        ("平野", "Hirano"),
        ("忍野八海", "Oshino Hakkai"),
        ("ふじさん牧場", "Fujisan Farm"),
        ("道の駅なるさわ", "Michi-no-Eki Narusawa"),
        ("富士緑の休暇村", "Fuji Midori-no-Kyukamura"),
        ("精進湖", "Shojiko"),
        ("本栖入口", "Motosuko Entrance"),
        ("本栖湖", "Motosuko"),
        ("富士山五合目", "Mt. Fuji 5th Station"),
        // === KOFU / YAMANASHI AREA ===
        ("甲府駅", "Kofu Station"),
        ("甲府駅南口", "Kofu Station South Exit"),
        ("甲府昭和インター", "Kofu Showa IC"),
        ("竜王", "Ryuo"),
        ("中央道双葉ＳＡ", "Chuo Expressway Futaba SA"),
        ("石和温泉駅", "Isawa Onsen Station"),
        ("石和", "Isawa"),
        ("山梨市駅", "Yamanashishi Station"),
        ("一宮", "Ichinomiya"),
        ("春日居", "Kasugai"),
        ("塩山駅", "Enzan Station"),
        ("勝沼ぶどう郷駅", "Katsunuma Budokyo Station"),
        ("勝沼インター", "Katsunuma IC"),
        // === MINOBU / SOUTH ALPS ===
        ("身延駅", "Minobu Station"),
        ("身延山", "Minobusan"),
        ("下部温泉", "Shimobe Onsen"),
        ("南アルプス市八田", "South Alps City Hatta"),
        ("六郷インター", "Rokugo IC"),
        ("増穂インター", "Masuho IC"),
        ("鰍沢口駅", "Kajikazawa-guchi Station"),
        // === SUWA / OKAYA / CHINO AREA ===
        ("諏訪インター前", "Suwa IC"),
        ("上諏訪駅", "Kamisuwa Station"),
        ("岡谷駅前", "Okaya Station"),
        ("茅野駅", "Chino Station"),
        ("諏訪湖ＳＡ", "Suwako SA"),
        ("原村", "Haramura"),
        ("蓼科高原", "Tateshina Kogen"),
        ("白樺湖", "Shirakabako"),
        ("車山高原", "Kurumayama Kogen"),
        // === INA / IIDA AREA ===
        ("伊那インター", "Ina IC"),
        ("伊那バスターミナル", "Ina Bus Terminal"),
        ("伊那市駅", "Inashi Station"),
        ("高遠駅", "Takato Station"),
        ("駒ヶ根インター", "Komagane IC"),
        ("駒ヶ根バスターミナル", "Komagane Bus Terminal"),
        ("飯田駅前", "Iida Station"),
        ("飯田インター", "Iida IC"),
        ("中央道伊那インター", "Chuo Expressway Ina IC"),
        ("中央道駒ヶ根", "Chuo Expressway Komagane"),
        ("中央道飯田", "Chuo Expressway Iida"),
        ("中央道松川", "Chuo Expressway Matsukawa"),
        ("中央道辰野", "Chuo Expressway Tatsuno"),
        ("箕輪", "Minowa"),
        // === MATSUMOTO AREA ===
        ("松本バスターミナル", "Matsumoto Bus Terminal"),
        ("松本インター前", "Matsumoto IC"),
        ("浅間温泉", "Asama Onsen"),
        ("美ヶ原温泉", "Utsukushigahara Onsen"),
        ("中央道岡谷", "Chuo Expressway Okaya"),
        ("塩尻北インター", "Shiojiri Kita IC"),
        ("みどり湖", "Midoriko"),
        ("広丘野村", "Hiroka Nomura"),
        ("村井", "Murai"),
        ("並柳", "Namiyanagi"),
        // === NAGANO / HAKUBA AREA ===
        ("長野駅", "Nagano Station"),
        ("長野駅東口", "Nagano Station East Exit"),
        ("川中島古戦場", "Kawanakajima Battlefield"),
        ("篠ノ井駅", "Shinonoi Station"),
        ("善光寺大門", "Zenkoji Daimon"),
        ("上田駅前", "Ueda Station"),
        ("上田菅平インター", "Ueda Sugadaira IC"),
        ("佐久平駅", "Sakudaira Station"),
        ("小諸インター", "Komoro IC"),
        ("白馬八方", "Hakuba Happo"),
        ("白馬五竜", "Hakuba Goryu"),
        ("栂池高原", "Tsugaike Kogen"),
        ("神城駅", "Kamishiro Station"),
        ("大町駅", "Omachi Station"),
        ("大町温泉郷", "Omachi Onsenkyo"),
        ("信濃大町駅", "Shinano-Omachi Station"),
        ("安曇野穂高", "Azumino Hotaka"),
        ("穂高駅", "Hotaka Station"),
        // === SHIOJIRI / KISO AREA ===
        ("塩尻駅前", "Shiojiri Station"),
        ("木曽福島駅", "Kiso-Fukushima Station"),
        ("木曽福島", "Kiso-Fukushima"),
        ("藪原", "Yabuhara"),
        ("奈良井", "Narai"),
        ("日義", "Hiyoshi"),
        ("木曽町役場", "Kisomachi Town Hall"),
        ("開田高原", "Kaida Kogen"),
        // === KAMIKOCHI / NORTHERN ALPS ===
        ("上高地バスターミナル", "Kamikochi Bus Terminal"),
        ("さわんどバスターミナル", "Sawando Bus Terminal"),
        ("新島々駅", "Shin-Shimashima Station"),
        ("大正池", "Taisho Pond"),
        ("帝国ホテル前", "Imperial Hotel Mae"),
        ("中の湯", "Nakanoyu"),
        ("乗鞍高原", "Norikura Kogen"),
        ("白骨温泉", "Shirahone Onsen"),
        ("平湯温泉", "Hirayu Onsen"),
        ("平湯バスターミナル", "Hirayu Bus Terminal"),
        ("新穂高ロープウェイ", "Shinhotaka Ropeway"),
        ("奥飛騨温泉郷", "Okuhida Onsenkyo"),
        // === KARUIZAWA AREA ===
        ("軽井沢駅", "Karuizawa Station"),
        ("軽井沢プリンスホテル", "Karuizawa Prince Hotel"),
        ("中軽井沢駅", "Naka-Karuizawa Station"),
        ("軽井沢72ゴルフ", "Karuizawa 72 Golf"),
        ("成田空港第１ターミナル", "Narita Airport Terminal 1"),
        ("成田空港第２ターミナル", "Narita Airport Terminal 2"),
        ("成田空港第３ターミナル", "Narita Airport Terminal 3"),
        // === TAKAYAMA / HIDA AREA ===
        ("高山濃飛バスセンター", "Takayama Nohi Bus Center"),
        ("高山バスセンター", "Takayama Bus Center"),
        ("高山駅前", "Takayama Station"),
        ("丹生川", "Nyukawa"),
        ("荘川", "Shokawa"),
        ("ひるがの高原", "Hirugano Kogen"),
        ("白川郷", "Shirakawa-go"),
        ("白川郷バスターミナル", "Shirakawa-go Bus Terminal"),
        ("五箇山", "Gokayama"),
        ("飛騨古川駅", "Hida-Furukawa Station"),
        ("飛騨清見インター", "Hida Kiyomi IC"),
        // === NAGOYA AREA ===
        ("名鉄バスセンター", "Meitetsu Bus Center"),
        ("名古屋駅新幹線口", "Nagoya Station Shinkansen Exit"),
        ("名古屋駅太閤通口", "Nagoya Station Taiko-dori Exit"),
        ("名古屋南ささしまライブ", "Nagoya Minami Sasashima Live"),
        ("栄オアシス21", "Sakae Oasis 21"),
        ("星ヶ丘", "Hoshigaoka"),
        ("藤が丘駅", "Fujigaoka Station"),
        ("尾張一宮駅前", "Owari Ichinomiya Station"),
        ("岐阜駅", "Gifu Station"),
        ("岐阜駅前", "Gifu Station"),
        ("名古屋インター", "Nagoya IC"),
        // === KANAZAWA / HOKURIKU AREA ===
        ("金沢駅", "Kanazawa Station"),
        ("金沢駅西口", "Kanazawa Station West Exit"),
        ("金沢駅東口", "Kanazawa Station East Exit"),
        ("香林坊", "Korinbo"),
        ("武蔵ヶ辻", "Musashigatsuji"),
        ("富山駅前", "Toyama Station"),
        ("富山インター", "Toyama IC"),
        ("高岡駅前", "Takaoka Station"),
        ("高岡インター", "Takaoka IC"),
        ("砺波駅前", "Tonami Station"),
        ("福井駅", "Fukui Station"),
        ("福井駅前", "Fukui Station"),
        ("福井インター", "Fukui IC"),
        ("鯖江インター", "Sabae IC"),
        ("敦賀インター", "Tsuruga IC"),
        // === GUJO / HIRUGANO AREA ===
        ("郡上八幡インター", "Gujo Hachiman IC"),
        ("郡上八幡駅", "Gujo Hachiman Station"),
        ("郡上白鳥駅", "Gujo Shirotori Station"),
        ("ひるがの高原ＳＡ", "Hirugano Kogen SA"),
        ("牧歌の里", "Bokka no Sato"),
        ("高鷲インター", "Takasu IC"),
        // === SHIZUOKA AREA ===
        ("三島駅", "Mishima Station"),
        ("三島駅北口", "Mishima Station North Exit"),
        ("沼津駅", "Numazu Station"),
        ("沼津駅北口", "Numazu Station North Exit"),
        ("清水駅前", "Shimizu Station"),
        ("静岡駅前", "Shizuoka Station"),
        ("静岡駅北口", "Shizuoka Station North Exit"),
        ("浜松駅", "Hamamatsu Station"),
        ("浜松駅前", "Hamamatsu Station"),
        ("浜松インター", "Hamamatsu IC"),
        ("磐田インター", "Iwata IC"),
        ("掛川インター", "Kakegawa IC"),
        ("御殿場駅", "Gotemba Station"),
        ("御殿場インター", "Gotemba IC"),
        ("御殿場プレミアムアウトレット", "Gotemba Premium Outlets"),
        ("裾野インター", "Susono IC"),
        // === SENDAI / TOHOKU AREA ===
        ("仙台駅", "Sendai Station"),
        ("仙台駅東口", "Sendai Station East Exit"),
        ("仙台駅前", "Sendai Station"),
        ("仙台宮城インター", "Sendai Miyagi IC"),
        ("石巻駅前", "Ishinomaki Station"),
        ("石巻営業所", "Ishinomaki Office"),
        ("気仙沼", "Kesennuma"),
        ("古川駅", "Furukawa Station"),
        ("鳴子温泉", "Naruko Onsen"),
        ("郡山駅前", "Koriyama Station"),
        ("郡山インター", "Koriyama IC"),
        ("福島駅前", "Fukushima Station"),
        ("宇都宮駅", "Utsunomiya Station"),
        ("宇都宮駅東口", "Utsunomiya Station East Exit"),
        ("那須塩原駅", "Nasushiobara Station"),
        ("佐野プレミアムアウトレット", "Sano Premium Outlets"),
        // === AOMORI / NORTHERN TOHOKU ===
        ("青森駅前", "Aomori Station"),
        ("青森フェリーターミナル", "Aomori Ferry Terminal"),
        ("弘前バスターミナル", "Hirosaki Bus Terminal"),
        ("弘前駅前", "Hirosaki Station"),
        ("八戸駅", "Hachinohe Station"),
        ("八戸中心街ターミナル", "Hachinohe Downtown Terminal"),
        ("三沢駅", "Misawa Station"),
        ("十和田市中央", "Towada City Center"),
        ("むつバスターミナル", "Mutsu Bus Terminal"),
        ("下北駅", "Shimokita Station"),
        ("大湊駅", "Ominato Station"),
        // === OSAKA / KANSAI AREA ===
        ("大阪梅田（阪急三番街）", "Osaka Umeda (Hankyu Sanban-gai)"),
        ("大阪駅前（東梅田駅）", "Osaka Station (Higashi-Umeda)"),
        ("なんばOCAT", "Namba OCAT"),
        ("天王寺駅", "Tennoji Station"),
        (
            "ユニバーサル・スタジオ・ジャパン",
            "Universal Studios Japan",
        ),
        ("ＵＳＪ", "USJ"),
        ("京都駅八条口", "Kyoto Station Hachijo Exit"),
        ("京都駅烏丸口", "Kyoto Station Karasuma Exit"),
        ("京都深草", "Kyoto Fukakusa"),
        ("神戸三宮", "Kobe Sannomiya"),
        ("神戸三宮バスターミナル", "Kobe Sannomiya Bus Terminal"),
        ("姫路駅", "Himeji Station"),
        ("姫路駅前", "Himeji Station"),
        // === SHIKOKU AREA ===
        ("高松駅", "Takamatsu Station"),
        ("高松駅高速バスターミナル", "Takamatsu Highway Bus Terminal"),
        ("丸亀駅", "Marugame Station"),
        ("坂出駅", "Sakaide Station"),
        ("善通寺インター", "Zentsuji IC"),
        ("徳島駅", "Tokushima Station"),
        ("徳島駅前", "Tokushima Station"),
        ("阿南駅", "Anan Station"),
        ("松山市駅", "Matsuyamashi Station"),
        ("松山駅前", "Matsuyama Station"),
        ("大街道", "Okaido"),
        ("道後温泉", "Dogo Onsen"),
        ("今治駅", "Imabari Station"),
        // === FUKUOKA / KYUSHU AREA ===
        ("博多バスターミナル", "Hakata Bus Terminal"),
        ("天神バスセンター", "Tenjin Bus Center"),
        (
            "西鉄天神高速バスターミナル",
            "Nishitetsu Tenjin Highway Bus Terminal",
        ),
        ("小倉駅前", "Kokura Station"),
        ("門司港駅", "Mojiko Station"),
        // === OKAYAMA AREA ===
        ("岡山駅", "Okayama Station"),
        ("岡山駅西口", "Okayama Station West Exit"),
        ("倉敷駅", "Kurashiki Station"),
        ("津山駅", "Tsuyama Station"),
        // === HANEDA AIRPORT ROUTES ===
        ("羽田空港第１ターミナル", "Haneda Airport Terminal 1"),
        ("羽田空港第２ターミナル", "Haneda Airport Terminal 2"),
        ("羽田空港第３ターミナル", "Haneda Airport Terminal 3"),
        ("羽田空港", "Haneda Airport"),
        // === TAMA AREA (Haneda routes) ===
        ("調布駅", "Chofu Station"),
        ("調布駅北口", "Chofu Station North Exit"),
        ("若葉台駅", "Wakabadai Station"),
        ("稲城駅", "Inagi Station"),
        ("京王永山駅", "Keio Nagayama Station"),
        ("京王多摩センター駅", "Keio Tama Center Station"),
        ("多摩センター駅", "Tama Center Station"),
        ("聖蹟桜ヶ丘駅", "Seiseki-Sakuragaoka Station"),
        ("国分寺駅", "Kokubunji Station"),
        ("国分寺駅南口", "Kokubunji Station South Exit"),
        ("武蔵小金井駅", "Musashi-Koganei Station"),
        ("武蔵小金井駅南口", "Musashi-Koganei Station South Exit"),
        ("府中駅", "Fuchu Station"),
        ("府中駅南口", "Fuchu Station South Exit"),
        ("八王子駅", "Hachioji Station"),
        ("八王子駅北口", "Hachioji Station North Exit"),
        ("八王子駅南口", "Hachioji Station South Exit"),
        ("京王八王子駅", "Keio Hachioji Station"),
        ("高尾駅", "Takao Station"),
        ("めじろ台駅", "Mejirodai Station"),
        ("西八王子駅", "Nishi-Hachioji Station"),
        // === WEST TOKYO (Haneda routes) ===
        ("河辺駅", "Kawabe Station"),
        ("河辺駅北口", "Kawabe Station North Exit"),
        ("羽村駅", "Hamura Station"),
        ("羽村駅東口", "Hamura Station East Exit"),
        ("福生駅", "Fussa Station"),
        ("福生駅西口", "Fussa Station West Exit"),
        ("秋川駅", "Akigawa Station"),
        ("秋川駅北口", "Akigawa Station North Exit"),
        ("あきる野インター", "Akiruno IC"),
        ("青梅駅", "Ome Station"),
        ("拝島駅", "Haijima Station"),
        ("昭島駅", "Akishima Station"),
        ("立川駅", "Tachikawa Station"),
        ("立川駅北口", "Tachikawa Station North Exit"),
        // === KUSATSU ONSEN ROUTE ===
        ("草津温泉バスターミナル", "Kusatsu Onsen Bus Terminal"),
        ("草津温泉", "Kusatsu Onsen"),
        ("長野原草津口駅", "Naganohara-Kusatsuguchi Station"),
        ("川原湯温泉駅", "Kawarayu Onsen Station"),
        ("八ッ場ダム", "Yamba Dam"),
        ("渋川駅", "Shibukawa Station"),
        ("前橋駅", "Maebashi Station"),
        ("高崎駅", "Takasaki Station"),
        // === SUMMERLAND ROUTE ===
        ("東京サマーランド", "Tokyo Summerland"),
        ("秋川駅", "Akigawa Station"),
        ("武蔵五日市駅", "Musashi-Itsukaichi Station"),
        // === SAGAMIKO ILLUMILLION ===
        ("さがみ湖イルミリオン", "Sagamiko Illumillion"),
        ("さがみ湖リゾート", "Sagamiko Resort"),
        ("相模湖駅", "Sagamiko Station"),
        ("高尾山口駅", "Takaosanguchi Station"),
        // === ADDITIONAL STATIONS ===
        ("東京ビッグサイト", "Tokyo Big Sight"),
        ("お台場", "Odaiba"),
        ("有明", "Ariake"),
        ("豊洲駅", "Toyosu Station"),
        ("新木場駅", "Shin-Kiba Station"),
        ("錦糸町駅", "Kinshicho Station"),
        ("秋葉原駅", "Akihabara Station"),
        ("上野駅", "Ueno Station"),
        ("浅草駅", "Asakusa Station"),
        ("千葉駅", "Chiba Station"),
        ("千葉中央駅", "Chiba-Chuo Station"),
        ("柏駅", "Kashiwa Station"),
        ("つくば駅", "Tsukuba Station"),
        ("筑波大学", "Tsukuba University"),
        ("水戸駅", "Mito Station"),
        ("日立駅", "Hitachi Station"),
        // === EXPRESSWAY SERVICE AREAS ===
        ("談合坂ＳＡ", "Dangozaka SA"),
        ("双葉ＳＡ", "Futaba SA"),
        ("諏訪湖ＳＡ", "Suwako SA"),
        ("駒ヶ岳ＳＡ", "Komagatake SA"),
        ("養老ＳＡ", "Yoro SA"),
        ("多賀ＳＡ", "Taga SA"),
        ("浜名湖ＳＡ", "Hamanako SA"),
        ("足柄ＳＡ", "Ashigara SA"),
        ("海老名ＳＡ", "Ebina SA"),
        ("港北ＰＡ", "Kohoku PA"),
        ("三芳ＰＡ", "Miyoshi PA"),
        // === MAGOME / TSUMAGO ===
        ("馬籠", "Magome"),
        ("妻籠", "Tsumago"),
        ("南木曽駅", "Nagiso Station"),
        ("中津川駅", "Nakatsugawa Station"),
        ("中津川インター", "Nakatsugawa IC"),
        ("恵那駅", "Ena Station"),
        ("恵那インター", "Ena IC"),
        ("恵那峡", "Enakyo"),
        ("瑞浪インター", "Mizunami IC"),
        ("多治見インター", "Tajimi IC"),
        ("土岐プレミアムアウトレット", "Toki Premium Outlets"),
    ])
});

pub fn translate_route_name(japanese: &str) -> String {
    ROUTE_NAMES
        .get(japanese)
        .map_or_else(|| japanese.to_string(), |s| (*s).to_string())
}

pub fn translate_station_name(japanese: &str) -> String {
    STATION_NAMES
        .get(japanese)
        .map_or_else(|| japanese.to_string(), |s| (*s).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ROUTE NAME TRANSLATION TESTS ===

    #[test]
    fn test_translate_route_name_known_shinjuku_fuji() {
        let result = translate_route_name("新宿～富士五湖線");
        assert_eq!(result, "Shinjuku - Fuji Five Lakes");
    }

    #[test]
    fn test_translate_route_name_known_nagoya_fukuoka() {
        let result = translate_route_name("名古屋～福岡線");
        assert_eq!(result, "Nagoya - Fukuoka");
    }

    #[test]
    fn test_translate_route_name_known_haneda_tama() {
        let result = translate_route_name("羽田多摩センター線");
        assert_eq!(result, "Haneda - Tama Center");
    }

    #[test]
    fn test_translate_route_name_unknown_returns_original() {
        let result = translate_route_name("未知の路線");
        assert_eq!(result, "未知の路線");
    }

    #[test]
    fn test_translate_route_name_empty_string() {
        let result = translate_route_name("");
        assert_eq!(result, "");
    }

    // === STATION NAME TRANSLATION TESTS ===

    #[test]
    fn test_translate_station_name_known_shinjuku() {
        let result = translate_station_name("バスタ新宿（南口）");
        assert_eq!(result, "Shinjuku Expressway Bus Terminal (South Exit)");
    }

    #[test]
    fn test_translate_station_name_known_kawaguchiko() {
        let result = translate_station_name("河口湖駅");
        assert_eq!(result, "Kawaguchiko Station");
    }

    #[test]
    fn test_translate_station_name_known_fuji_highland() {
        let result = translate_station_name("富士急ハイランド");
        assert_eq!(result, "Fuji-Q Highland");
    }

    #[test]
    fn test_translate_station_name_known_haneda() {
        let result = translate_station_name("羽田空港第１ターミナル");
        assert_eq!(result, "Haneda Airport Terminal 1");
    }

    #[test]
    fn test_translate_station_name_unknown_returns_original() {
        let result = translate_station_name("未知の駅");
        assert_eq!(result, "未知の駅");
    }

    #[test]
    fn test_translate_station_name_empty_string() {
        let result = translate_station_name("");
        assert_eq!(result, "");
    }

    // === LAZYLOCKS INITIALIZATION TESTS ===

    #[test]
    fn test_route_names_map_is_not_empty() {
        assert!(!ROUTE_NAMES.is_empty());
        assert!(ROUTE_NAMES.len() >= 50); // Should have at least 50 routes
    }

    #[test]
    fn test_station_names_map_is_not_empty() {
        assert!(!STATION_NAMES.is_empty());
        assert!(STATION_NAMES.len() >= 200); // Should have at least 200 stations
    }

    #[test]
    fn test_route_names_sample_entries_exist() {
        // Verify key entries from different areas
        assert!(ROUTE_NAMES.contains_key("新宿～富士五湖線")); // Area 1
        assert!(ROUTE_NAMES.contains_key("名古屋～福岡線")); // Area 2
        assert!(ROUTE_NAMES.contains_key("羽田多摩センター線")); // Area 3
        assert!(ROUTE_NAMES.contains_key("新宿～松本線"));
        assert!(ROUTE_NAMES.contains_key("新宿～名古屋線"));
    }

    #[test]
    fn test_station_names_sample_entries_exist() {
        // Verify key stations from different categories
        assert!(STATION_NAMES.contains_key("バスタ新宿（南口）")); // Tokyo terminal
        assert!(STATION_NAMES.contains_key("河口湖駅")); // Fuji area
        assert!(STATION_NAMES.contains_key("名鉄バスセンター")); // Nagoya area
        assert!(STATION_NAMES.contains_key("金沢駅")); // Hokuriku
        assert!(STATION_NAMES.contains_key("羽田空港第１ターミナル")); // Airport
        assert!(STATION_NAMES.contains_key("草津温泉バスターミナル")); // Onsen
    }

    #[test]
    fn test_all_route_translations_are_non_empty() {
        for (jp, en) in ROUTE_NAMES.iter() {
            assert!(!jp.is_empty(), "Japanese route name should not be empty");
            assert!(!en.is_empty(), "English route name should not be empty");
        }
    }

    #[test]
    fn test_all_station_translations_are_non_empty() {
        for (jp, en) in STATION_NAMES.iter() {
            assert!(!jp.is_empty(), "Japanese station name should not be empty");
            assert!(!en.is_empty(), "English station name should not be empty");
        }
    }
}
