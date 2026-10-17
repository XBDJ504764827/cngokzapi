-- 地图制作人 + 服务器实时详情（配合首页公告/地图卡片/服务器卡片视图，见 PROJECT §2）。
-- 幂等：全部 IF NOT EXISTS。

-- 地图制作人显示名：上游 gokz.top 只给作者 SteamID，制作人名称取自 kzgo 地图元数据，
-- 因此单独一列，缺失时为空数组（前端显示"作者未知"）。
ALTER TABLE maps ADD COLUMN IF NOT EXISTS mappers TEXT[] NOT NULL DEFAULT '{}';

-- 服务器：当前地图难度（按地图名从 maps 关联）+ 实时在线玩家昵称列表。
ALTER TABLE servers
    ADD COLUMN IF NOT EXISTS map_tier     SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS player_names TEXT[] NOT NULL DEFAULT '{}';
