// ========== 排行榜模拟数据生成（占位，后续替换为外部挖掘的真实数据） ==========
// 真实数据接入时只需替换 buildMockLeaderboard 的返回值，条目字段保持兼容：
// { rank, player, steamId64, avatar, mapsCompleted, totalPoints, avgPoints, records, lastPlayed, lastPlayedDaysAgo }
const LEADERBOARD_SIZE = 120;
const LEADERBOARD_MODE_INDEX = { KZT: 0, SKZ: 1, VNL: 2 };
const LEADERBOARD_HEAD = {
    KZT: [
        { player: 'ProGamer', mapsCompleted: 234, totalPoints: 45678, avgPoints: 195.2, records: 567 },
        { player: 'SpeedRunner', mapsCompleted: 221, totalPoints: 43210, avgPoints: 195.5, records: 534 },
        { player: 'TechMaster', mapsCompleted: 198, totalPoints: 39456, avgPoints: 199.3, records: 489 }
    ],
    SKZ: [
        { player: 'JumpGod', mapsCompleted: 198, totalPoints: 38456, avgPoints: 194.2, records: 512 },
        { player: 'ProGamer', mapsCompleted: 187, totalPoints: 35678, avgPoints: 190.8, records: 478 },
        { player: 'BunnyhopKing', mapsCompleted: 176, totalPoints: 33456, avgPoints: 190.1, records: 423 }
    ],
    VNL: [
        { player: 'TechMaster', mapsCompleted: 176, totalPoints: 32456, avgPoints: 184.4, records: 445 },
        { player: 'KZ_Master', mapsCompleted: 165, totalPoints: 30123, avgPoints: 182.6, records: 412 },
        { player: 'ClimbFan', mapsCompleted: 154, totalPoints: 28789, avgPoints: 186.9, records: 387 }
    ]
};
const LEADERBOARD_NAME_POOL = [
    'JumpGod', 'BunnyhopKing', 'StrafePro', 'KZ_Master', 'ClimbFan', 'NewbieMaster',
    'AirControl', 'StrafeKing', 'BhopGod', 'KZLegend', 'SurfPro', 'ClimbExpert',
    'TriggerHappy', 'EdgeBug', 'LongJump', 'Bhopper', 'SurfMaster', 'KreedzFan',
    'TimerHunter', 'Checkpoint', 'NoScope', 'PreStrafe', 'W Strafe', 'SlideMaster',
    'CrouchBug', 'LadderKing', 'RampSlayer', 'BlockMaster', 'GapJumper', 'TechNinja'
];

// 确定性伪随机，保证每次打开榜单一致，方便对照验证
function mulberry32(seed) {
    let a = seed >>> 0;
    return function () {
        a |= 0;
        a = (a + 0x6D2B79F5) | 0;
        let t = Math.imul(a ^ (a >>> 15), 1 | a);
        t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

// 模拟头像：与站内其他头像同源（dicebear identicon），后续替换为 Steam 真实头像 URL
function mockAvatar(seedName) {
    return 'https://api.dicebear.com/7.x/identicon/svg?seed=' + encodeURIComponent(seedName);
}

// 模拟 SteamID64：17 位、76561198 开头，后续替换为真实 SteamID64
function mockSteamId64(mode, i) {
    const num = 100000000 + LEADERBOARD_MODE_INDEX[mode] * 20000000 + i * 7919;
    return '76561198' + String(num).padStart(9, '0');
}

function daysAgoToDate(days) {
    const d = new Date();
    d.setDate(d.getDate() - days);
    return d.toISOString().slice(0, 10);
}

function formatDaysAgo(days) {
    if (days <= 0) return '今天';
    if (days === 1) return '昨天';
    return days + '天前';
}

function buildMockLeaderboard(mode) {
    const rng = mulberry32(1000 + LEADERBOARD_MODE_INDEX[mode] * 97);
    const head = LEADERBOARD_HEAD[mode] || LEADERBOARD_HEAD.KZT;
    const rows = head.map((h, i) => {
        const days = Math.floor(rng() * 10); // 头部玩家默认近 10 天活跃
        return {
            rank: i + 1,
            player: h.player,
            steamId64: mockSteamId64(mode, i),
            avatar: mockAvatar(mode + ':' + h.player),
            mapsCompleted: h.mapsCompleted,
            totalPoints: h.totalPoints,
            avgPoints: h.avgPoints,
            records: h.records,
            lastPlayedDaysAgo: days,
            lastPlayed: daysAgoToDate(days)
        };
    });
    let points = head[head.length - 1].totalPoints;
    // 生成池剔除头部已用名称，避免重名（各模式头部不同，需按模式过滤）
    const pool = LEADERBOARD_NAME_POOL.filter((n) => !head.some((h) => h.player === n));
    for (let i = head.length; i < LEADERBOARD_SIZE; i++) {
        const idx = i - head.length;
        const base = pool[idx % pool.length];
        const suffix = idx >= pool.length ? '_' + (Math.floor(idx / pool.length) + 1) : '';
        const name = base + suffix;
        points = Math.max(1000, points - Math.floor(80 + rng() * 320));
        const maps = Math.max(5, Math.floor(rows[i - 1].mapsCompleted - rng() * 4));
        const avg = Math.round((150 + rng() * 48) * 10) / 10;
        const records = Math.max(12, Math.floor(rows[i - 1].records - rng() * 6));
        const days = Math.floor(rng() * 61); // 0~60 天前，保证各活跃度筛选项都有数据
        rows.push({
            rank: i + 1,
            player: name,
            steamId64: mockSteamId64(mode, i),
            avatar: mockAvatar(mode + ':' + name),
            mapsCompleted: maps,
            totalPoints: points,
            avgPoints: avg,
            records: records,
            lastPlayedDaysAgo: days,
            lastPlayed: daysAgoToDate(days)
        });
    }
    return rows;
}

// ========== 模拟数据 ==========
const mockData = {
    servers: [
        {
            id: 1,
            name: '【CN】KZ 中国服务器 #1',
            map: 'kz_hopstep',
            mapImage: 'https://placehold.co/600x400/2F6F4F/F8FAFC?text=kz_hopstep',
            ip: '192.168.1.100:27015',
            players: ['Player1', 'Player2', 'ProGamer', 'KZ_Master'],
            maxPlayers: 20,
            region: '中国',
            tier: 3
        },
        {
            id: 2,
            name: '【CN】KZ 练习服 #2',
            map: 'kz_easy_beta',
            mapImage: 'https://placehold.co/600x400/4C8F6B/F8FAFC?text=kz_easy_beta',
            ip: '192.168.1.101:27016',
            players: ['NewbieMaster', 'JumpGod'],
            maxPlayers: 16,
            region: '中国',
            tier: 1
        },
        {
            id: 3,
            name: '【CN】KZ 竞技服 #3',
            map: 'kz_extreme_tower',
            mapImage: 'https://placehold.co/600x400/B5533C/F8FAFC?text=kz_extreme_tower',
            ip: '192.168.1.102:27017',
            players: ['ProPlayer', 'SpeedRunner', 'TechMaster', 'BunnyhopKing', 'StrafePro'],
            maxPlayers: 20,
            region: '中国',
            tier: 7
        }
    ],
    
    recentRecords: [
        { player: 'ProGamer', map: 'kz_hopstep', mode: 'KZT', time: '01:23.45', points: 892, date: '2024-01-15' },
        { player: 'KZ_Master', map: 'kz_hopstep', mode: 'KZT', time: '01:25.67', points: 884, date: '2024-01-14' },
        { player: 'StrafePro', map: 'kz_bhop_easy', mode: 'KZT', time: '00:36.78', points: 845, date: '2024-01-13' },
        { player: 'SpeedRunner', map: 'kz_easy_beta', mode: 'SKZ', time: '00:45.23', points: 678, date: '2024-01-15' },
        { player: 'JumpGod', map: 'kz_bhop_easy', mode: 'SKZ', time: '00:34.12', points: 723, date: '2024-01-13' },
        { player: 'BunnyhopKing', map: 'kz_hopstep', mode: 'SKZ', time: '01:31.09', points: 701, date: '2024-01-12' },
        { player: 'TechMaster', map: 'kz_extreme_tower', mode: 'VNL', time: '03:12.87', points: 956, date: '2024-01-14' },
        { player: 'ClimbFan', map: 'kz_climb_hard', mode: 'VNL', time: '02:41.55', points: 902, date: '2024-01-13' },
        { player: 'NewbieMaster', map: 'kz_easy_beta', mode: 'VNL', time: '00:58.31', points: 640, date: '2024-01-12' }
    ],

    // KZT / SKZ / VNL 三个独立排行榜(各模式独立计分、独立排序，每榜 120 人模拟数据)
    leaderboards: {
        KZT: buildMockLeaderboard('KZT'),
        SKZ: buildMockLeaderboard('SKZ'),
        VNL: buildMockLeaderboard('VNL')
    },
    
    maps: [
        { id: 1, name: 'kz_hopstep', tier: 3, author: 'MapMaker', type: '线性', completions: 1234, avgTime: '01:45.32', worldRecord: '01:23.45', image: 'https://placehold.co/800x450/2F6F4F/F8FAFC?text=kz_hopstep' },
        { id: 2, name: 'kz_easy_beta', tier: 1, author: 'BeginnerCreator', type: '线性', completions: 2456, avgTime: '00:52.18', worldRecord: '00:34.12', image: 'https://placehold.co/800x450/4C8F6B/F8FAFC?text=kz_easy_beta' },
        { id: 3, name: 'kz_extreme_tower', tier: 7, author: 'HardcoreMapper', type: '垂直', completions: 234, avgTime: '04:23.76', worldRecord: '03:12.87', image: 'https://placehold.co/800x450/B5533C/F8FAFC?text=kz_extreme_tower' },
        { id: 4, name: 'kz_bhop_easy', tier: 2, author: 'BhopMaster', type: 'Bhop', completions: 1876, avgTime: '00:48.92', worldRecord: '00:31.45', image: 'https://placehold.co/800x450/4C8F6B/F8FAFC?text=kz_bhop_easy' },
        { id: 5, name: 'kz_surf_complex', tier: 5, author: 'SurfPro', type: 'Surf', completions: 567, avgTime: '02:34.56', worldRecord: '01:54.23', image: 'https://placehold.co/800x450/B8862F/F8FAFC?text=kz_surf_complex' },
        { id: 6, name: 'kz_climb_hard', tier: 6, author: 'ClimbExpert', type: '垂直', completions: 432, avgTime: '03:45.89', worldRecord: '02:34.12', image: 'https://placehold.co/800x450/B5533C/F8FAFC?text=kz_climb_hard' }
    ],
    
    // 单张地图在各模式下的 TOP 榜(各模式独立)
    mapLeaderboards: {
        KZT: [
            { rank: 1, player: 'ProGamer', time: '01:23.45', points: 892, date: '2024-01-15' },
            { rank: 2, player: 'SpeedRunner', time: '01:24.12', points: 889, date: '2024-01-14' },
            { rank: 3, player: 'TechMaster', time: '01:25.78', points: 883, date: '2024-01-13' }
        ],
        SKZ: [
            { rank: 1, player: 'JumpGod', time: '01:19.87', points: 901, date: '2024-01-15' },
            { rank: 2, player: 'ProGamer', time: '01:21.33', points: 895, date: '2024-01-14' },
            { rank: 3, player: 'BunnyhopKing', time: '01:22.56', points: 890, date: '2024-01-12' }
        ],
        VNL: [
            { rank: 1, player: 'TechMaster', time: '01:27.91', points: 878, date: '2024-01-14' },
            { rank: 2, player: 'KZ_Master', time: '01:29.44', points: 871, date: '2024-01-13' },
            { rank: 3, player: 'ClimbFan', time: '01:31.02', points: 865, date: '2024-01-12' }
        ]
    },
    
    // 玩家在各模式下的统计(跟随全站模式总开关,各模式独立)
    profileStats: {
        KZT: { rank: 1, points: 45678, mapsCompleted: 234, totalRecords: 567, avgPoints: 195.2 },
        SKZ: { rank: 3, points: 35678, mapsCompleted: 187, totalRecords: 478, avgPoints: 190.8 },
        VNL: { rank: 8, points: 28789, mapsCompleted: 154, totalRecords: 387, avgPoints: 186.9 }
    },

    // 玩家在各模式下完成的地图(引用 maps 的 id,各模式不同)
    profileMaps: {
        KZT: [1, 3, 5, 6],
        SKZ: [2, 4, 1, 5],
        VNL: [3, 6, 5, 2]
    },

    announcements: [
        { id: 1, title: '欢迎来到 CNGOKZ!', content: '感谢各位玩家的支持,平台正式上线运营。服务器接入说明详见 API 文档。', date: '2024-01-15' },
        { id: 2, title: '新增 VNL 模式支持', content: 'VNL 模式排行榜已上线,所有记录将独立计分。', date: '2024-01-10' },
        { id: 3, title: '反作弊系统升级', content: '平台加强了记录审核机制,可疑成绩将进入人工审核队列。', date: '2024-01-05' }
    ]
};

// ========== 全局状态 ==========
const MODES = ['KZT', 'SKZ', 'VNL'];
let currentMode = 'KZT';  // 全站模式总开关,默认 KZT
let serverViewMode = 'list';  // 服务器视图模式: grid / list，默认列式（列表）
let currentMapId = null;  // 当前打开的地图详情 ID(用于模式切换时刷新榜单)

// 模式总开关:点击循环切换 KZT → SKZ → VNL → KZT
function cycleMode() {
    const next = MODES[(MODES.indexOf(currentMode) + 1) % MODES.length];
    setMode(next);
}

function setMode(mode) {
    if (!MODES.includes(mode)) return;
    currentMode = mode;
    const switchBtn = document.getElementById('modeSwitch');
    if (switchBtn) switchBtn.textContent = mode;
    const lbLabel = document.getElementById('leaderboardModeLabel');
    if (lbLabel) lbLabel.textContent = mode;
    const recLabel = document.getElementById('recordsModeLabel');
    if (recLabel) recLabel.textContent = mode;
    const mapLabel = document.getElementById('mapModeLabel');
    if (mapLabel) mapLabel.textContent = mode;
    const profileLabel = document.getElementById('profileModeLabel');
    if (profileLabel) profileLabel.textContent = mode;
    // 刷新所有跟模式相关的榜单与个人资料
    renderLeaderboard(mode);
    renderRecentRecords();
    renderMapLeaderboard();
    renderProfile();
}

// ========== 页面导航 ==========
const pages = ['home', 'records', 'leaderboard', 'maps', 'servers', 'mapDetail', 'profile'];

function navigateTo(pageName) {
    pages.forEach(page => {
        const pageEl = document.getElementById(page + 'Page');
        if (pageEl) pageEl.classList.remove('active');
    });
    
    const targetPage = document.getElementById(pageName + 'Page');
    if (targetPage) targetPage.classList.add('active');
    
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.remove('active');
        if (link.getAttribute('data-page') === pageName) {
            link.classList.add('active');
        }
    });
}

// ========== 渲染函数 ==========
function renderAnnouncements() {
    const list = document.getElementById('announcementList');
    if (!list) return;
    list.innerHTML = mockData.announcements.map(a => `
        <div class="announcement-card">
            <div class="announcement-header">
                <h3 class="announcement-title">${a.title}</h3>
                <span class="announcement-date">${a.date}</span>
            </div>
            <p class="announcement-content">${a.content}</p>
        </div>
    `).join('');
}

function renderRecentRecords() {
    const recordsTable = document.getElementById('recentRecords');
    if (!recordsTable) return;
    const rows = mockData.recentRecords.filter(r => r.mode === currentMode);
    recordsTable.innerHTML = rows.map(record => `
        <tr>
            <td><a href="#" class="player-link" onclick="navigateTo('profile'); return false;">${record.player}</a></td>
            <td>${record.map}</td>
            <td><span class="mode-badge">${record.mode}</span></td>
            <td class="time-value">${record.time}</td>
            <td class="points-value">${record.points}</td>
            <td>${record.date}</td>
        </tr>
    `).join('');
}

// ========== 排行榜状态（搜索 / 筛选 / 排序，筛选条件在模式切换时保留） ==========
let leaderboardQuery = '';
let leaderboardActivity = 'all'; // all | 7 | 30 | inactive
let leaderboardSort = 'totalPoints'; // totalPoints | mapsCompleted | avgPoints | records

function getFilteredLeaderboard(mode) {
    const all = mockData.leaderboards[mode] || mockData.leaderboards.KZT;
    const q = leaderboardQuery.trim().toLowerCase();
    let rows = all.filter(entry => {
        // 名称模糊匹配（不区分大小写）或 SteamID64 子串匹配
        if (q && !(entry.player.toLowerCase().includes(q) || entry.steamId64.includes(q))) return false;
        if (leaderboardActivity === '7' && entry.lastPlayedDaysAgo > 7) return false;
        if (leaderboardActivity === '30' && entry.lastPlayedDaysAgo > 30) return false;
        if (leaderboardActivity === 'inactive' && entry.lastPlayedDaysAgo <= 30) return false;
        return true;
    });
    const key = leaderboardSort;
    rows = rows.slice().sort((a, b) => b[key] - a[key]);
    return { all, rows };
}

function renderLeaderboard(mode = 'KZT') {
    const leaderboardData = document.getElementById('leaderboardData');
    if (!leaderboardData) return;

    const { all, rows } = getFilteredLeaderboard(mode);
    const countEl = document.getElementById('leaderboardCount');
    if (countEl) {
        countEl.textContent = rows.length === all.length
            ? '共 ' + all.length + ' 人'
            : '筛出 ' + rows.length + ' 人 / 共 ' + all.length + ' 人';
    }
    const emptyEl = document.getElementById('leaderboardEmpty');
    if (emptyEl) emptyEl.hidden = rows.length !== 0;

    leaderboardData.innerHTML = rows.map(entry => {
        let rankClass = '';
        if (entry.rank === 1) rankClass = 'top-1';
        else if (entry.rank === 2) rankClass = 'top-2';
        else if (entry.rank === 3) rankClass = 'top-3';

        return `
            <tr>
                <td><span class="rank-number ${rankClass}">#${entry.rank}</span></td>
                <td>
                    <div class="lb-player">
                        <img class="lb-avatar" src="${entry.avatar}" alt="${entry.player} 的头像" loading="lazy" onerror="this.style.visibility='hidden'">
                        <div class="lb-id">
                            <a href="#" class="player-link" onclick="navigateTo('profile'); return false;">${entry.player}</a>
                            <span class="lb-steam">${entry.steamId64}</span>
                        </div>
                    </div>
                </td>
                <td>${entry.mapsCompleted}</td>
                <td class="points-value">${entry.totalPoints.toLocaleString()}</td>
                <td>${entry.avgPoints.toFixed(1)}</td>
                <td>${entry.records}</td>
                <td><span class="lb-last-active">${formatDaysAgo(entry.lastPlayedDaysAgo)}</span><span class="lb-last-date">${entry.lastPlayed}</span></td>
            </tr>
        `;
    }).join('');
}

// 外部入口：导航栏搜索框复用排行榜搜索（按名称 / SteamID64），并跳到排行榜页
function applyLeaderboardSearch(query) {
    leaderboardQuery = query || '';
    const input = document.getElementById('leaderboardSearch');
    if (input && input.value !== leaderboardQuery) input.value = leaderboardQuery;
    renderLeaderboard(currentMode);
}

function renderMaps() {
    const mapsGrid = document.getElementById('mapsGrid');
    if (!mapsGrid) return;
    mapsGrid.innerHTML = mockData.maps.map(map => `
        <div class="map-card" onclick="showMapDetail(${map.id})">
            <div class="map-thumbnail" style="background-image: url('${map.image}')"></div>
            <div class="map-content">
                <div class="map-title">${map.name}</div>
                <div class="map-meta-row">
                    <span class="tier-badge tier-${map.tier}">Tier ${map.tier}</span>
                    <span class="map-author-text">${map.author}</span>
                </div>
                <div class="map-stats">
                    <div class="map-stat">
                        <div class="map-stat-value">${map.completions}</div>
                        <div class="map-stat-label">完成次数</div>
                    </div>
                    <div class="map-stat">
                        <div class="map-stat-value">${map.worldRecord}</div>
                        <div class="map-stat-label">世界纪录</div>
                    </div>
                </div>
            </div>
        </div>
    `).join('');
}

// 继续在下一个块...
function renderServers(viewMode = 'grid') {
    const container = document.getElementById('serversContainer');
    if (!container) return;

    container.className = viewMode === 'grid' ? 'servers-grid' : 'servers-list';

    const playerChips = (players) => players.map(p => `<span class="player-chip">${p}</span>`).join('');

    if (viewMode === 'grid') {
        container.innerHTML = mockData.servers.map(server => `
            <div class="server-card-grid">
                <div class="server-map-preview" style="background-image: url('${server.mapImage}')">
                    <div class="server-status-badge">
                        <span class="status-dot"></span> 在线
                    </div>
                </div>
                <div class="server-card-content">
                    <h3 class="server-name">${server.name}</h3>
                    <div class="server-info-row">
                        <span class="server-info-label">当前地图</span>
                        <span class="server-info-value">${server.map}</span>
                    </div>
                    <div class="server-info-row">
                        <span class="server-info-label">难度 / 区域</span>
                        <span class="server-info-value">Tier ${server.tier} · ${server.region}</span>
                    </div>
                    <div class="server-info-row">
                        <span class="server-info-label">玩家</span>
                        <span class="server-info-value">${server.players.length}/${server.maxPlayers}</span>
                    </div>
                    <div class="server-players">
                        <div class="server-players-title">在线玩家</div>
                        <div class="server-players-list">${playerChips(server.players)}</div>
                    </div>
                    <div class="server-actions">
                        <button class="btn-action btn-copy" onclick="copyServerIP('${server.ip}')">
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                            </svg>
                            复制 IP
                        </button>
                        <button class="btn-action btn-join" onclick="joinServer('${server.ip}')">
                            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M13.8 12H3"/>
                            </svg>
                            加入
                        </button>
                    </div>
                </div>
            </div>
        `).join('');
    } else {
        container.innerHTML = mockData.servers.map(server => `
            <div class="server-card-list">
                <div class="server-list-preview" style="background-image: url('${server.mapImage}')">
                    <div class="server-status-badge">
                        <span class="status-dot"></span> 在线
                    </div>
                </div>
                <div class="server-list-info">
                    <h3 class="server-name">${server.name}</h3>
                    <div class="server-meta">
                        <span>${server.map}</span>
                        <span>·</span>
                        <span>Tier ${server.tier}</span>
                        <span>·</span>
                        <span>${server.region}</span>
                        <span>·</span>
                        <span>${server.players.length}/${server.maxPlayers} 玩家</span>
                    </div>
                    <div class="server-players-list">${playerChips(server.players)}</div>
                </div>
                <div class="server-list-actions">
                    <button class="btn-action btn-copy" onclick="copyServerIP('${server.ip}')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                        </svg>
                        复制 IP
                    </button>
                    <button class="btn-action btn-join" onclick="joinServer('${server.ip}')">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4M10 17l5-5-5-5M13.8 12H3"/>
                        </svg>
                        加入
                    </button>
                </div>
            </div>
        `).join('');
    }
}

function copyServerIP(ip) {
    const done = () => alert(`服务器 IP 已复制: ${ip}`);
    const fallback = () => {
        try {
            const ta = document.createElement('textarea');
            ta.value = ip;
            ta.style.position = 'fixed';
            ta.style.opacity = '0';
            document.body.appendChild(ta);
            ta.select();
            document.execCommand('copy');
            document.body.removeChild(ta);
            done();
        } catch (e) {
            alert(`复制失败,请手动复制: ${ip}`);
        }
    };
    if (navigator.clipboard && window.isSecureContext !== false) {
        navigator.clipboard.writeText(ip).then(done).catch(fallback);
    } else {
        fallback();
    }
}

function joinServer(ip) {
    // 用隐藏 iframe 拉起 steam:// 协议,避免整页跳转导致原型页面丢失
    const iframe = document.createElement('iframe');
    iframe.style.display = 'none';
    iframe.src = `steam://connect/${ip}`;
    document.body.appendChild(iframe);
    setTimeout(() => document.body.removeChild(iframe), 2000);
    alert(`正在拉起 Steam 加入服务器: ${ip}\n\n如果没有自动打开,请确保已安装 Steam 客户端,或手动复制 IP 在控制台连接。`);
}

function showMapDetail(mapId) {
    const map = mockData.maps.find(m => m.id === mapId);
    if (!map) return;
    currentMapId = mapId;

    document.getElementById('mapDetailImage').style.backgroundImage = `url('${map.image}')`;
    document.getElementById('mapName').textContent = map.name;
    document.getElementById('mapTier').textContent = `Tier ${map.tier}`;
    document.getElementById('mapAuthor').textContent = map.author;
    document.getElementById('mapType').textContent = map.type;
    document.getElementById('mapCompletions').textContent = map.completions.toLocaleString();
    document.getElementById('mapAvgTime').textContent = map.avgTime;
    document.getElementById('mapWR').textContent = map.worldRecord;

    renderMapLeaderboard();

    navigateTo('mapDetail');
}

// 地图详情 TOP 榜:跟随全站模式总开关
function renderMapLeaderboard() {
    const leaderboardData = document.getElementById('mapLeaderboardData');
    if (!leaderboardData) return;
    const mapLabel = document.getElementById('mapModeLabel');
    if (mapLabel) mapLabel.textContent = currentMode;
    const data = mockData.mapLeaderboards[currentMode] || mockData.mapLeaderboards.KZT;
    leaderboardData.innerHTML = data.map(entry => {
        let rankClass = '';
        if (entry.rank === 1) rankClass = 'top-1';
        else if (entry.rank === 2) rankClass = 'top-2';
        else if (entry.rank === 3) rankClass = 'top-3';

        return `
            <tr>
                <td><span class="rank-number ${rankClass}">#${entry.rank}</span></td>
                <td><a href="#" class="player-link" onclick="navigateTo('profile'); return false;">${entry.player}</a></td>
                <td class="time-value">${entry.time}</td>
                <td class="points-value">${entry.points}</td>
                <td>${entry.date}</td>
            </tr>
        `;
    }).join('');
}

function renderProfile() {
    // 取当前模式下的玩家统计
    const stats = mockData.profileStats[currentMode] || mockData.profileStats.KZT;
    
    document.getElementById('profileName').textContent = 'ProGamer';
    const profileModeLabel = document.getElementById('profileModeLabel');
    if (profileModeLabel) profileModeLabel.textContent = currentMode;
    document.getElementById('profileRank').textContent = `#${stats.rank}`;
    document.getElementById('profileSteamID').textContent = 'STEAM_0:1:12345678';
    document.getElementById('profilePoints').textContent = stats.points.toLocaleString();
    document.getElementById('profileMapsCompleted').textContent = stats.mapsCompleted;
    document.getElementById('profileTotalRecords').textContent = stats.totalRecords;
    document.getElementById('profileAvgPoints').textContent = stats.avgPoints.toFixed(1);
    
    // 该模式下完成的地图
    const completedIds = mockData.profileMaps[currentMode] || mockData.profileMaps.KZT;
    const completedMaps = completedIds.map(id => mockData.maps.find(m => m.id === id)).filter(Boolean);

    const recentMaps = document.getElementById('profileRecentMaps');
    if (recentMaps) {
        recentMaps.innerHTML = completedMaps.map(map => `
            <div class="map-card" onclick="showMapDetail(${map.id})">
                <div class="map-thumbnail" style="background-image: url('${map.image}')"></div>
                <div class="map-content">
                    <div class="map-title">${map.name}</div>
                    <div class="map-meta-row">
                        <span class="tier-badge tier-${map.tier}">Tier ${map.tier}</span>
                    </div>
                </div>
            </div>
        `).join('');
    }
    
    const recordsData = document.getElementById('profileRecordsData');
    if (recordsData) {
        recordsData.innerHTML = mockData.recentRecords.filter(r => r.mode === currentMode).slice(0, 5).map(record => `
            <tr>
                <td>${record.map}</td>
                <td><span class="mode-badge">${record.mode}</span></td>
                <td class="time-value">${record.time}</td>
                <td><span class="rank-number">#1</span></td>
                <td class="points-value">${record.points}</td>
                <td>${record.date}</td>
            </tr>
        `).join('');
    }
}

// ========== 事件监听 ==========
document.addEventListener('DOMContentLoaded', () => {
    // 导航链接
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', (e) => {
            e.preventDefault();
            navigateTo(link.getAttribute('data-page'));
        });
    });
    
    // Steam 登录
    const loginBtn = document.getElementById('loginBtn');
    const userChip = document.getElementById('userChip');
    if (loginBtn) {
        loginBtn.addEventListener('click', () => {
            alert('正在跳转到 Steam 登录页面...\n(这是模拟功能,实际需要集成 Steam OpenID)');
            loginBtn.style.display = 'none';
            userChip.hidden = false;
            document.getElementById('userChipName').textContent = 'ProGamer';
        });
    }
    
    if (userChip) {
        userChip.addEventListener('click', () => {
            renderProfile();
            navigateTo('profile');
        });
    }
    
    // 导航栏玩家搜索：按名称 / SteamID64 在排行榜中定位，不再是弹窗模拟
    const searchInput = document.getElementById('playerSearchInput');
    if (searchInput) {
        searchInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                const query = searchInput.value.trim();
                if (query) {
                    applyLeaderboardSearch(query);
                    navigateTo('leaderboard');
                    searchInput.value = '';
                }
            }
        });
    }

    // 排行榜页内搜索 / 筛选 / 排序
    const lbSearch = document.getElementById('leaderboardSearch');
    if (lbSearch) {
        lbSearch.addEventListener('input', (e) => {
            leaderboardQuery = e.target.value;
            renderLeaderboard(currentMode);
        });
    }
    const lbActivity = document.getElementById('leaderboardActivityFilter');
    if (lbActivity) {
        lbActivity.addEventListener('change', (e) => {
            leaderboardActivity = e.target.value;
            renderLeaderboard(currentMode);
        });
    }
    const lbSort = document.getElementById('leaderboardSortFilter');
    if (lbSort) {
        lbSort.addEventListener('change', (e) => {
            leaderboardSort = e.target.value;
            renderLeaderboard(currentMode);
        });
    }
    
    // 模式总开关:点击循环切换 KZT → SKZ → VNL
    const modeSwitch = document.getElementById('modeSwitch');
    if (modeSwitch) {
        modeSwitch.addEventListener('click', cycleMode);
    }
    
    // 服务器视图切换
    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.view-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            const view = btn.getAttribute('data-view');
            serverViewMode = view;
            renderServers(view);
        });
    });
    
    // 初始化渲染
    renderAnnouncements();
    renderRecentRecords();
    renderLeaderboard(currentMode);
    renderMaps();
    renderServers(serverViewMode);
});

// 全局暴露函数
window.navigateTo = navigateTo;
window.showMapDetail = showMapDetail;
window.copyServerIP = copyServerIP;
window.joinServer = joinServer;
window.cycleMode = cycleMode;
window.setMode = setMode;
window.applyLeaderboardSearch = applyLeaderboardSearch;
