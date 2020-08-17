output "lab_instance_id" {
  value = join(",", aws_instance.lab.*.id)
}

output "lab_public_dns" {
  value = join(",", aws_instance.lab.*.public_dns)
}

output "lab_public_ip" {
  value = join(",", aws_instance.lab.*.public_ip)
}

output "bucket_target" {
  value = aws_s3_bucket.target_bucket.bucket
}

output "bucket_attacker" {
  value = aws_s3_bucket.attacker_bucket.bucket
}
