// 首页「最新公告」内容。
// 目前为静态数据（与原原型 docs/prototype/cngokz-web 的公告一致）；
// 后续接管理后台/接口时，替换为接口数据即可（PROJECT §10 下一步）。

export interface Announcement {
  id: number
  title: string
  content: string
  date: string
}

export const announcements: Announcement[] = [
  {
    id: 1,
    title: '欢迎来到 CNGOKZ!',
    content: '感谢各位玩家的支持,平台正式上线运营。服务器接入说明详见 API 文档。',
    date: '2024-01-15',
  },
  {
    id: 2,
    title: '新增 VNL 模式支持',
    content: 'VNL 模式排行榜已上线,所有记录将独立计分。',
    date: '2024-01-10',
  },
  {
    id: 3,
    title: '反作弊系统升级',
    content: '平台加强了记录审核机制,可疑成绩将进入人工审核队列。',
    date: '2024-01-05',
  },
]
