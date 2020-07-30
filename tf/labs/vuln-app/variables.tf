variable "aws_region" {
  type = string
}

variable "vulnerable_ami_id" {
  type = string
}

variable "vuln_app_pub_key" {
  default = "ami/vuln_app.pub"
}

variable "vuln_app_pri_key" {
  default = "ami/vuln_app"
}

variable "key_name" {
  default = "vuln-app-key"
}

variable "instance_type" {
  default = "t2.nano"
}
