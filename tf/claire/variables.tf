variable "aws_profile" {
  description = "The AWS Profile to use to deploy CLAIRE"
  default     = "default"
}

variable "aws_region" {
  description = "The AWS region to use to the deploy CLAIRE"
  type        = string
}

variable "vpc_id" {
  description = "The VPC to be used to create security groups"
  type        = string
}

variable "allowed_cidr_when_lockeddown" {
  description = "If SSM VPC endpoints are enabled this can be the VPC CIDR otherwise 0.0.0.0/0"
  type        = list(string)
}

variable "prefix" {
  description = "The prefix for s3 buckets (for uniqueness)"
  type        = string
}

variable "evidence_extractor_ami_id" {
  description = "The Evidence Extractor AMI created by packer"
  type        = string
}

variable "enable_guardduty" {
  description = "Enable GuardDuty and setup to trigger CLAIRE"
  type        = number
}

variable "guardduty_alert_thresholds" {
  description = "Which levels will trigger CLAIRE"
  type        = string
  default     = "[5.0, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 6.0, 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 7.0, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 7.9, 8.0, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8, 8.9]"
}

variable "instance_isolation_threshold" {
  description = "The minium severity required to isolate the instance"
  default     = 7
}
