use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

use crate::shared::error::AppError;

const MAX_LOOKAHEAD_SECONDS: i64 = 366 * 24 * 60 * 60;

#[derive(Debug, Clone)]
struct CronSpec {
    seconds: CronField,
    minutes: CronField,
    hours: CronField,
    days: CronField,
    months: CronField,
    weekdays: CronField,
}

#[derive(Debug, Clone)]
struct CronField {
    min: u32,
    max: u32,
    any: bool,
    values: Vec<u32>,
}

pub fn validate_cron_expression(expression: &str) -> Result<(), AppError> {
    parse_cron_expression(expression).map(|_| ())
}

pub fn next_fire_time(expression: &str, after: DateTime<Utc>) -> Result<DateTime<Utc>, AppError> {
    let spec = parse_cron_expression(expression)?;
    let mut candidate = after + Duration::seconds(1);
    let deadline = after + Duration::seconds(MAX_LOOKAHEAD_SECONDS);

    while candidate <= deadline {
        if spec.matches(candidate) {
            return Ok(candidate);
        }
        candidate += Duration::seconds(1);
    }

    Err(AppError::bad_request(
        "cron 表达式在一年内没有下一次触发时间",
    ))
}

fn parse_cron_expression(expression: &str) -> Result<CronSpec, AppError> {
    let parts = expression.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(AppError::bad_request(
            "cron 表达式必须包含 6 段（秒 分 时 日 月 周）",
        ));
    }

    Ok(CronSpec {
        seconds: CronField::parse(parts[0], 0, 59, false)?,
        minutes: CronField::parse(parts[1], 0, 59, false)?,
        hours: CronField::parse(parts[2], 0, 23, false)?,
        days: CronField::parse(parts[3], 1, 31, true)?,
        months: CronField::parse(parts[4], 1, 12, false)?,
        weekdays: CronField::parse(parts[5], 0, 7, true)?,
    })
}

impl CronSpec {
    fn matches(&self, value: DateTime<Utc>) -> bool {
        self.seconds.matches(value.second())
            && self.minutes.matches(value.minute())
            && self.hours.matches(value.hour())
            && self.days.matches(value.day())
            && self.months.matches(value.month())
            && self.matches_weekday(value)
    }

    fn matches_weekday(&self, value: DateTime<Utc>) -> bool {
        let weekday = value.weekday().num_days_from_sunday();
        self.weekdays.matches(weekday) || (weekday == 0 && self.weekdays.matches(7))
    }
}

impl CronField {
    fn parse(raw: &str, min: u32, max: u32, allow_question: bool) -> Result<Self, AppError> {
        let raw = raw.trim();
        if raw == "*" || (allow_question && raw == "?") {
            return Ok(Self {
                min,
                max,
                any: true,
                values: Vec::new(),
            });
        }
        if raw.is_empty() {
            return Err(AppError::bad_request("cron 字段不能为空"));
        }

        let mut values = Vec::new();
        for segment in raw.split(',') {
            push_segment_values(segment.trim(), min, max, &mut values)?;
        }
        values.sort_unstable();
        values.dedup();
        if values.is_empty() {
            return Err(AppError::bad_request("cron 字段没有可用取值"));
        }

        Ok(Self {
            min,
            max,
            any: false,
            values,
        })
    }

    fn matches(&self, value: u32) -> bool {
        if self.any {
            return value >= self.min && value <= self.max;
        }
        self.values.binary_search(&value).is_ok()
    }
}

fn push_segment_values(
    segment: &str,
    min: u32,
    max: u32,
    values: &mut Vec<u32>,
) -> Result<(), AppError> {
    if segment.is_empty() {
        return Err(AppError::bad_request("cron 字段包含空片段"));
    }

    let (base, step) = if let Some((base, step)) = segment.split_once('/') {
        let step = parse_number(step, min, max)?;
        if step == 0 {
            return Err(AppError::bad_request("cron 步长必须大于 0"));
        }
        (base, step)
    } else {
        (segment, 1)
    };

    let (start, end) = if base == "*" {
        (min, max)
    } else if let Some((start, end)) = base.split_once('-') {
        (parse_number(start, min, max)?, parse_number(end, min, max)?)
    } else {
        let value = parse_number(base, min, max)?;
        (value, value)
    };

    if start > end {
        return Err(AppError::bad_request("cron 范围起始值不能大于结束值"));
    }

    let mut current = start;
    while current <= end {
        values.push(current);
        current = match current.checked_add(step) {
            Some(next) => next,
            None => break,
        };
    }

    Ok(())
}

fn parse_number(raw: &str, min: u32, max: u32) -> Result<u32, AppError> {
    let value = raw
        .parse::<u32>()
        .map_err(|_| AppError::bad_request("cron 字段必须是数字、*、范围或步长"))?;
    if value < min || value > max {
        return Err(AppError::bad_request(format!(
            "cron 字段取值必须在 {min} 到 {max} 之间"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    #[test]
    fn next_fire_time_supports_second_level_cron() {
        let after = Utc.with_ymd_and_hms(2026, 5, 30, 12, 0, 0).unwrap();

        let next = next_fire_time("*/10 * * * * *", after).unwrap();

        assert_eq!(next.second(), 10);
        assert_eq!(next.timestamp(), after.timestamp() + 10);
    }

    #[test]
    fn validate_cron_expression_rejects_invalid_input() {
        let err = validate_cron_expression("not a cron").unwrap_err();

        assert!(err.to_string().contains("cron"));
    }
}
