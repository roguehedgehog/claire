variable aws_profile {
  default = "default"
}

variable aws_region {
  type = string
}

variable "prefix" {
  type = string
}

variable "evidence_extractor_ami_id" {
  type = string
}
