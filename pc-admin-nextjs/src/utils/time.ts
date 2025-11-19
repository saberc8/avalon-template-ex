/**
 * 根据当前时间返回问候语
 */
export function goodTimeText(): string {
  const time = new Date();
  const hour = time.getHours();
  return hour < 9
    ? "早上好"
    : hour <= 11
    ? "上午好"
    : hour <= 13
    ? "中午好"
    : hour <= 18
    ? "下午好"
    : "晚上好";
}
