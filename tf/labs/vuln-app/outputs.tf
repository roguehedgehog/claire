output "vuln_app_instance_id" {
  value = aws_instance.vuln_app.id
}

output "vuln_app_public_dns" {
  value = aws_instance.vuln_app.public_dns
}

output "vuln_app_public_ip" {
  value = aws_instance.vuln_app.public_ip
}
