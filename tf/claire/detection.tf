resource "aws_guardduty_detector" "detector" {
  count  = var.enable_guardduty
  enable = true
}

resource "aws_cloudwatch_event_rule" "detection_rule" {
  count = var.enable_guardduty
  name  = "claire_lab_guardduty_alerts"

  event_pattern = <<PATTERN
{
    "source": [
      "aws.guardduty"
    ],
    "detail-type": [
      "GuardDuty Finding"
    ],
    "detail": {
      "severity": ${var.guardduty_alert_thresholds}
    }
}
PATTERN
}

resource "aws_cloudwatch_event_target" "detection_target" {
  count = var.enable_guardduty

  rule     = aws_cloudwatch_event_rule.detection_rule[0].name
  arn      = aws_sfn_state_machine.claire_investigation.arn
  role_arn = aws_iam_role.claire_state_machine_role.arn

}
