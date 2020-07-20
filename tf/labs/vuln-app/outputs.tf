output "vuln_app_instance_id" {
  value = join("", aws_instance.vuln_app.*.id, aws_instance.vuln_app_provisioned.*.id)
}
output "vuln_app_public_dns" {
  value = join("", aws_instance.vuln_app.*.public_dns, aws_instance.vuln_app_provisioned.*.public_dns)
}

output "vuln_app_public_ip" {
  value = join("", aws_instance.vuln_app.*.public_ip, aws_instance.vuln_app_provisioned.*.public_ip)
}