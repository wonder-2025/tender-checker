#!/usr/bin/env node

/**
 * 许可证生成工具
 * 
 * 用法：
 *   node license-generator.js <设备指纹> <用户名> <公司> <邮箱> <有效期天数> <每日限制>
 * 
 * 示例：
 *   node license-generator.js abc123 张三 "XX科技公司" test@example.com 365 50
 */

const crypto = require('crypto');

// 生成UUID
function generateUUID() {
    return crypto.randomUUID();
}

// 生成许可证ID
function generateLicenseId() {
    const year = new Date().getFullYear();
    const shortId = generateUUID().split('-')[0].toUpperCase();
    return `TC-${year}-${shortId}`;
}

// 生成许可证
function generateLicense(deviceFingerprint, userName, company, email, daysValid, maxChecksPerDay, features) {
    const now = new Date();
    const expiresAt = new Date(now.getTime() + daysValid * 24 * 60 * 60 * 1000);
    
    const license = {
        license_id: generateLicenseId(),
        license_key: '',
        device_fingerprint: deviceFingerprint,
        user_name: userName,
        company: company,
        email: email,
        expires_at: expiresAt.toISOString(),
        max_checks_per_day: maxChecksPerDay,
        features: features || ['basic_check', 'tender_parse', 'custom_rules', 'export_report'],
        created_at: now.toISOString(),
        signature: ''
    };
    
    // Base64编码
    const jsonStr = JSON.stringify(license, null, 2);
    const licenseKey = Buffer.from(jsonStr).toString('base64');
    
    return licenseKey;
}

// 命令行参数
const args = process.argv.slice(2);

if (args.length < 6) {
    console.log(`
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   标书智能检查工具 - 许可证生成器
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

用法：
  node license-generator.js <设备指纹> <用户名> <公司> <邮箱> <有效期天数> <每日限制>

参数说明：
  设备指纹    - 目标设备的指纹（从工具中获取）
  用户名      - 授权用户姓名
  公司        - 公司名称（需用引号包裹）
  邮箱        - 联系邮箱
  有效期天数  - 许可证有效期（如365表示1年）
  每日限制    - 每日检查次数上限（如50）

示例：
  node license-generator.js abc123 张三 "XX科技公司" test@example.com 365 50

功能列表：
  - basic_check     : 基础检查功能
  - tender_parse    : 招标文件解析
  - custom_rules    : 自定义检查规则
  - export_report   : 报告导出
`);
    process.exit(1);
}

const [
    deviceFingerprint,
    userName,
    company,
    email,
    daysValid,
    maxChecksPerDay
] = args;

const licenseKey = generateLicense(
    deviceFingerprint,
    userName,
    company,
    email,
    parseInt(daysValid),
    parseInt(maxChecksPerDay)
);

console.log(`
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   许可证生成成功
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

授权信息：
  用户名      : ${userName}
  公司        : ${company}
  邮箱        : ${email}
  有效期      : ${daysValid} 天
  每日限制    : ${maxChecksPerDay} 次
  设备指纹    : ${deviceFingerprint}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   许可证密钥（复制以下内容）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

${licenseKey}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

使用方法：
1. 打开工具，进入"设置"页面
2. 点击"导入许可证"
3. 粘贴上面的许可证密钥
4. 点击确认

注意：许可证与设备绑定，不可用于其他设备。
`);
