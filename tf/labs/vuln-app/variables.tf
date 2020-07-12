variable "aws_region" {
  default = "eu-west-1"
}

variable "vulnerable_ami_id" {
  default = "ami-089cc16f7f08c4457"
}

variable "vuln_app_pub_key" {
  default = "res/vuln_app.pub"
}

variable "vuln_app_pri_key" {
  default = "res/vuln_app"
}

variable "vuln_app_init_script" {
  default = "res/install-vuln-drupal.sh"
}

variable "vuln_app_init_sql_gz" {
  default = "res/vuln_app.sql.gz"
}

variable "vuln_app_db" {
  default = "vuln_app"
}

variable "vuln_app_name" {
  default = "vuln_app"
}

variable "key_name" {
  default = "vuln-app-key"
}

variable "instance_type" {
  default = "t2.nano"
}