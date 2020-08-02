variable "aws_profile" {
  default = "default"
}

variable "aws_region" {
  type = string
}

variable "vpc_id" {
  type = string
}

variable "allowed_cidr_when_lockeddown" {
  type = list(string)
}

variable "prefix" {
  type = string
}

variable "evidence_extractor_ami_id" {
  type = string
}
