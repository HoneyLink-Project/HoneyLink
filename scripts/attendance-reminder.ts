#!/usr/bin/env node

/**
 * HoneyLink Stakeholder Attendance Reminder System
 *
 * Purpose: Automated reminder and escalation system to maintain 90%+ attendance
 * Integration: Slack Workflow + GitHub Issues
 *
 * Non-negotiables:
 * - No C/C++ dependencies (pure Node.js/TypeScript)
 * - Follows spec/notes/attendance-system.md
 * - Idempotent execution (can run multiple times safely)
 */

import { Octokit } from '@octokit/rest';
import { WebClient } from '@slack/web-api';
import * as dotenv from 'dotenv';

dotenv.config();

// ==================== Configuration ====================

interface WorkingGroup {
  name: string;
  slackChannel: string;
  chairRoleId: string;
  coreMembers: string[]; // Slack User IDs
  meetingSchedule: {
    dayOfWeek: number; // 0=Sunday, 1=Monday, ...
    hour: number; // JST hour (0-23)
    minute: number;
  };
}

const WORKING_GROUPS: WorkingGroup[] = [
  {
    name: 'Architecture WG',
    slackChannel: '#honeylink-wg-architecture',
    chairRoleId: '@arch-chair',
    coreMembers: ['U01234567', 'U01234568', 'U01234569'], // Placeholder IDs
    meetingSchedule: { dayOfWeek: 2, hour: 14, minute: 0 }, // Tuesday 14:00 JST
  },
  {
    name: 'Protocol WG',
    slackChannel: '#honeylink-wg-protocol',
    chairRoleId: '@protocol-chair',
    coreMembers: ['U01234570', 'U01234571', 'U01234572'],
    meetingSchedule: { dayOfWeek: 3, hour: 15, minute: 0 }, // Wednesday 15:00 JST
  },
  {
    name: 'UX WG',
    slackChannel: '#honeylink-wg-ux',
    chairRoleId: '@ux-chair',
    coreMembers: ['U01234573', 'U01234574', 'U01234575'],
    meetingSchedule: { dayOfWeek: 4, hour: 10, minute: 0 }, // Thursday 10:00 JST
  },
  {
    name: 'Security WG',
    slackChannel: '#honeylink-wg-security',
    chairRoleId: '@security-chair',
    coreMembers: ['U01234576', 'U01234577', 'U01234578'],
    meetingSchedule: { dayOfWeek: 2, hour: 16, minute: 0 }, // Tuesday 16:00 JST
  },
  {
    name: 'Operations WG',
    slackChannel: '#honeylink-wg-operations',
    chairRoleId: '@ops-chair',
    coreMembers: ['U01234579', 'U01234580', 'U01234581'],
    meetingSchedule: { dayOfWeek: 5, hour: 13, minute: 0 }, // Friday 13:00 JST
  },
];

// ==================== Slack Integration ====================

const slackClient = new WebClient(process.env.SLACK_BOT_TOKEN);

async function send48HourReminder(wg: WorkingGroup, meetingDate: Date): Promise<void> {
  const message = `📅 [HoneyLink WG] ${wg.name} 定例会議リマインダー (48時間前)

日時: ${formatDateJST(meetingDate)}
場所: <会議リンクをここに挿入>
アジェンダ: https://github.com/HoneyLink-Project/HoneyLink/blob/main/spec/notes/meeting-notes.md

欠席の場合は24時間前までに ${wg.chairRoleId} へ連絡してください。
補完レビューフォーム: https://github.com/HoneyLink-Project/HoneyLink/issues/new?template=補完レビュー計画書.md

📌 出席率90%を目標にご協力お願いします！`;

  try {
    await slackClient.chat.postMessage({
      channel: wg.slackChannel,
      text: message,
      mrkdwn: true,
    });
    console.log(`✅ 48h reminder sent to ${wg.slackChannel}`);
  } catch (error) {
    console.error(`❌ Failed to send 48h reminder to ${wg.slackChannel}:`, error);
  }
}

async function send24HourReminderToNonResponders(
  wg: WorkingGroup,
  nonResponders: string[]
): Promise<void> {
  for (const userId of nonResponders) {
    const message = `👋 [HoneyLink] ${wg.name} 会議出席確認 (24時間前)

明日の会議に出席できますか？

✅ 出席: このメッセージに反応してください
❌ 欠席: 補完レビュー計画書を提出してください
   フォーム: https://github.com/HoneyLink-Project/HoneyLink/issues/new?template=補完レビュー計画書.md

24時間以内に応答がない場合、無連絡欠席として記録されます。`;

    try {
      await slackClient.chat.postMessage({
        channel: userId, // DM
        text: message,
        mrkdwn: true,
      });
      console.log(`✅ 24h DM sent to ${userId}`);
    } catch (error) {
      console.error(`❌ Failed to send 24h DM to ${userId}:`, error);
    }
  }
}

async function send2HourFinalReminder(wg: WorkingGroup, meetingDate: Date): Promise<void> {
  const message = `⏰ [HoneyLink WG] ${wg.name} 会議開始2時間前！

日時: ${formatDateJST(meetingDate)}
場所: <会議リンクをここに挿入>

皆様のご参加をお待ちしております 🚀`;

  try {
    await slackClient.chat.postMessage({
      channel: wg.slackChannel,
      text: message,
      mrkdwn: true,
    });
    console.log(`✅ 2h final reminder sent to ${wg.slackChannel}`);
  } catch (error) {
    console.error(`❌ Failed to send 2h reminder to ${wg.slackChannel}:`, error);
  }
}

// ==================== GitHub Issues Integration ====================

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

async function createEscalationIssue(
  wg: WorkingGroup,
  absentees: string[],
  month: string
): Promise<void> {
  const title = `[Escalation] ${wg.name} 出席率90%未達 (${month})`;
  const body = `## エスカレーション通知

**対象ワーキンググループ**: ${wg.name}
**対象月**: ${month}
**出席率**: <算出値を入力>
**無連絡欠席メンバー**: ${absentees.join(', ')}

### 問題
${wg.name} の出席率が90%を下回りました。spec/notes/attendance-system.md Section 5.2 に従い、エスカレーションプロセスを開始します。

### アクション
- [ ] Chair (${wg.chairRoleId}) が欠席理由をヒアリング (Level 1)
- [ ] 必要に応じて Project Lead が介入 (Level 2)
- [ ] 構造的問題の場合は Steering Committee へエスカレーション (Level 3)

### 参照
- [Attendance System](https://github.com/HoneyLink-Project/HoneyLink/blob/main/spec/notes/attendance-system.md)
- [Governance](https://github.com/HoneyLink-Project/HoneyLink/blob/main/spec/notes/governance.md)

**Priority**: P1 (高)
**Due Date**: <1週間以内>`;

  try {
    const { data: issue } = await octokit.issues.create({
      owner: 'HoneyLink-Project',
      repo: 'HoneyLink',
      title,
      body,
      labels: ['escalation', 'governance', wg.name.toLowerCase().replace(' ', '-')],
      assignees: [wg.chairRoleId.replace('@', '')], // Remove @ prefix
    });
    console.log(`✅ Escalation issue created: ${issue.html_url}`);
  } catch (error) {
    console.error(`❌ Failed to create escalation issue:`, error);
  }
}

async function trackAttendance(wg: WorkingGroup, attendees: string[]): Promise<void> {
  const attendanceRate = (attendees.length / wg.coreMembers.length) * 100;
  const month = new Date().toISOString().slice(0, 7); // YYYY-MM

  if (attendanceRate < 90) {
    const absentees = wg.coreMembers.filter((member) => !attendees.includes(member));
    await createEscalationIssue(wg, absentees, month);
  }

  console.log(`📊 ${wg.name} attendance: ${attendanceRate.toFixed(1)}% (${attendees.length}/${wg.coreMembers.length})`);
}

// ==================== Scheduling Logic ====================

function getNextMeeting(wg: WorkingGroup): Date {
  const now = new Date();
  const jstOffset = 9 * 60; // JST = UTC+9
  const utcNow = new Date(now.getTime() + now.getTimezoneOffset() * 60000);
  const jstNow = new Date(utcNow.getTime() + jstOffset * 60000);

  let nextMeeting = new Date(jstNow);
  nextMeeting.setHours(wg.meetingSchedule.hour, wg.meetingSchedule.minute, 0, 0);

  // Find next occurrence of target day of week
  const daysUntilMeeting = (wg.meetingSchedule.dayOfWeek + 7 - jstNow.getDay()) % 7;
  nextMeeting.setDate(nextMeeting.getDate() + (daysUntilMeeting === 0 && jstNow > nextMeeting ? 7 : daysUntilMeeting));

  return nextMeeting;
}

function shouldSend48HourReminder(meetingDate: Date): boolean {
  const now = new Date();
  const timeDiff = meetingDate.getTime() - now.getTime();
  const hoursDiff = timeDiff / (1000 * 60 * 60);

  // Send reminder if meeting is 47-49 hours away (1-hour window for cron execution)
  return hoursDiff >= 47 && hoursDiff <= 49;
}

function shouldSend24HourReminder(meetingDate: Date): boolean {
  const now = new Date();
  const timeDiff = meetingDate.getTime() - now.getTime();
  const hoursDiff = timeDiff / (1000 * 60 * 60);

  return hoursDiff >= 23 && hoursDiff <= 25;
}

function shouldSend2HourReminder(meetingDate: Date): boolean {
  const now = new Date();
  const timeDiff = meetingDate.getTime() - now.getTime();
  const hoursDiff = timeDiff / (1000 * 60 * 60);

  return hoursDiff >= 1.5 && hoursDiff <= 2.5;
}

// ==================== Utility Functions ====================

function formatDateJST(date: Date): string {
  const jstDate = new Date(date.toLocaleString('en-US', { timeZone: 'Asia/Tokyo' }));
  const days = ['日', '月', '火', '水', '木', '金', '土'];
  const dayName = days[jstDate.getDay()];
  return `${jstDate.toISOString().slice(0, 10)} (${dayName}) ${jstDate.getHours()}:${String(jstDate.getMinutes()).padStart(2, '0')} JST`;
}

// ==================== Main Execution ====================

async function main(): Promise<void> {
  console.log('🚀 HoneyLink Attendance Reminder System starting...');
  console.log(`Current time: ${new Date().toISOString()}`);

  for (const wg of WORKING_GROUPS) {
    const nextMeeting = getNextMeeting(wg);
    console.log(`\n📅 ${wg.name} next meeting: ${formatDateJST(nextMeeting)}`);

    if (shouldSend48HourReminder(nextMeeting)) {
      console.log('  ⏰ Sending 48-hour reminder...');
      await send48HourReminder(wg, nextMeeting);
    } else if (shouldSend24HourReminder(nextMeeting)) {
      console.log('  ⏰ Sending 24-hour DM reminders...');
      // Placeholder: In production, fetch non-responders from database
      const nonResponders = wg.coreMembers.slice(0, 1); // Demo: assume 1 non-responder
      await send24HourReminderToNonResponders(wg, nonResponders);
    } else if (shouldSend2HourReminder(nextMeeting)) {
      console.log('  ⏰ Sending 2-hour final reminder...');
      await send2HourFinalReminder(wg, nextMeeting);
    } else {
      console.log('  ⏸️  No reminder scheduled at this time.');
    }
  }

  console.log('\n✅ Attendance reminder system completed successfully.');
}

// Run if executed directly
if (require.main === module) {
  main().catch((error) => {
    console.error('❌ Fatal error:', error);
    process.exit(1);
  });
}

export { createEscalationIssue, main, trackAttendance };
