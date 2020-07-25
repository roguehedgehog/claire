data "archive_file" "lambda_zip" {
  type        = "zip"
  output_path = "./dist/lambdas.zip"
  source_dir  = "../../bin/ir"
}

resource "aws_lambda_function" "create_investigation" {
  function_name = "claire_create_investigation"
  handler       = "create_investigation.lambda_handler"
  role          = module.create_investigation_role.arn

  environment {
    variables = {
      INVESTIGATION_BUCKET = aws_s3_bucket.investigation_bucket.bucket
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

}

resource "aws_lambda_function" "snapshot_disks" {
  function_name = "claire_snapshot_disks"
  handler       = "snapshot_disks.lambda_snapshot_handler"
  role          = module.snapshot_disks_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256

}

resource "aws_lambda_function" "isolate_instance" {
  function_name = "claire_isolate_instance"
  handler       = "isolate_instance.lambda_handler"
  role          = module.isolate_instance_role.arn

  environment {
    variables = {
      LOCKED_DOWN_SECURITY_GROUP = aws_security_group.locked_down.id
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "create_evidence_extractor" {
  function_name = "claire_create_evidence_extractor"
  handler       = "create_evidence_extractor.lambda_handler"
  role          = module.create_instance_role.arn

  environment {
    variables = {
      IAM_PROFILE = aws_iam_instance_profile.claire_ec2_evidence_extractor_profile.arn
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "poll_evidence_extractor" {
  function_name = "claire_poll_evidence_extractor"
  handler       = "create_evidence_extractor.lambda_is_extractor_ready"
  role          = module.poll_instance_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "terminate_evidence_extractor" {
  function_name = "claire_terminate_evidence_extractor"
  handler       = "create_evidence_extractor.lambda_terminate_extractor"
  role          = module.terminate_instance_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "prepare_memory_volume" {
  function_name = "claire_prepare_memory_volume"
  handler       = "extract_memory.lambda_prepare_memory_volume"
  role          = module.run_ssm_command_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "detach_volumes" {
  function_name = "claire_detach_volumes"
  handler       = "manage_volumes.lambda_detach_volumes"
  role          = module.manage_volume_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "attach_volumes" {
  function_name = "claire_attach_volumes"
  handler       = "manage_volumes.lambda_attach_volumes"
  role          = module.manage_volume_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "destroy_volumes" {
  function_name = "claire_destroy_volumes"
  handler       = "manage_volumes.lambda_destroy_volumes"
  role          = module.manage_volume_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}


resource "aws_lambda_function" "is_detach_volumes_complete" {
  function_name = "claire_is_detach_volumes_complete"
  handler       = "manage_volumes.lambda_is_detach_complete"
  role          = module.query_volume_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "is_attach_volumes_complete" {
  function_name = "claire_is_attach_volumes_complete"
  handler       = "manage_volumes.lambda_is_attach_complete"
  role          = module.query_volume_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "capture_memory" {
  function_name = "claire_capture_memory"
  handler       = "extract_memory.lambda_capture_memory"
  role          = module.run_ssm_command_role.arn

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "upload_memory" {
  function_name = "claire_upload_memory"
  handler       = "extract_memory.lambda_upload_memory"
  role          = module.run_ssm_command_role.arn

  environment {
    variables = {
      INVESTIGATION_BUCKET = aws_s3_bucket.investigation_bucket.bucket
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}

resource "aws_lambda_function" "is_command_complete" {
  function_name = "claire_is_command_complete"
  handler       = "run_command.lambda_is_command_complete"
  role          = module.query_ssm_command_role.arn

  environment {
    variables = {
      INVESTIGATION_BUCKET = aws_s3_bucket.investigation_bucket.bucket
    }
  }

  publish          = true
  runtime          = "python3.8"
  filename         = data.archive_file.lambda_zip.output_path
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}
